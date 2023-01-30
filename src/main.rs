use walkdir::WalkDir;


use std::io::BufRead;
use std::path::Path;
use std::fs;
use models::*;
use cli::*;
use regex::Regex;

mod models;
mod cli;


//cargo run -- -c /Users/sanj/ziptemp/renaming/series/murdoch-mysteries-81670/season-5.json -d /Volumes/MediaDrive/TV_Rips -r /Volumes/MediaDrive/TV
fn main() {
  let config = get_cli_args();
  println!("config: {:?}", config);
  let config_file = config.config_file;
  let dvd_rips_directory = config.dvd_rips;
  let renames_directory = config.renames_directory;

  let config_file_path = Path::new(&config_file);
  let dvd_rips_directory_path = Path::new(&dvd_rips_directory);
  let renames_directory_path = Path::new(&renames_directory);

  if !(config_file_path.exists() && dvd_rips_directory_path.exists() && renames_directory_path.exists()) {
      println!("One or more supplied file paths do not exist:");
      print_error_if_file_not_found("config_file", config_file_path);
      print_error_if_file_not_found("dvd_rips_directory", dvd_rips_directory_path);
      print_error_if_file_not_found("renames_directory", renames_directory_path)
  } else {
    let series_metadata = get_series_metadata(EpisodeGuide(config_file_path.to_owned()));
    println!("metadata: {:?}", series_metadata);
    println!("done")
  }
}

fn print_error_if_file_not_found(name: &str, p: &Path) {
  if !p.exists() {
    println!(" - Path for {} does not exist: {:?}", name, p)
  }
}

fn program(series_metadata: &SeriesMetaData, dvd_rips: &DvdRipsDir, renames_dir: &RenamesDir) {

  // TODO: Pass this in
  let dvd_rips_directory =  &dvd_rips.0; //"/Volumes/MediaDrive/TV_Rips"; //current dir

  // TODO: Pass this in
  let renames_directory = &renames_dir.0;//"/Volumes/MediaDrive/TV";

  // TODO: Pass this in via config file or read it from TVDB
  let episode_names =
    vec![
      Episode::new("S05E01", "Murdoch of the Klondike", "81670"),
      Episode::new("S05E02", "Back and to the Left", "81670"),
      Episode::new("S05E03", "Evil Eye of Egypt", "81670"),
      Episode::new("S05E04", "War on Terror", "81670"),
      Episode::new("S05E05", "Murdoch at the Opera", "81670"),
      Episode::new("S05E06", "Who Killed the Electric Carriage?", "81670"),
      Episode::new("S05E07", "Stroll on the Wild Side (1)", "81670"),
      Episode::new("S05E08", "Stroll on the Wild Side (2)", "81670"),
      Episode::new("S05E09", "Invention Convention", "81670"),
      Episode::new("S05E10", "Staircase to Heaven", "81670"),
      Episode::new("S05E11", "Murdoch in Toyland", "81670"),
      Episode::new("S05E12", "Murdoch Night in Canada", "81670"),
      Episode::new("S05E13", "Twentieth Century Murdoch", "81670"),
    ];

  let mut dirs: Vec<FileNameAndExt> = WalkDir::new(dvd_rips_directory)
      .into_iter()
      .filter_map(|re| re.ok())
      .filter_map(|dir_entry| {
        let p = dir_entry.path();
        let is_file = p.is_file();
        let has_disk_subdirectory = p.to_string_lossy().to_string().contains("/disk");
        if is_file && has_disk_subdirectory {
          p.file_name().and_then(|name|{
            p.extension().map(|ext| FileNameAndExt::new(p, name, ext))  // Some(FileNameAndExt)
          })
        } else {
          None
        }
     })
    .collect();

  dirs.sort_by(|fne1, fne2| fne1.partial_cmp(&fne2).unwrap());

  if dirs.len() > episode_names.len() {
    println!("Not enough Episode names ({}) to match actual files extracted ({})", episode_names.len(), dirs.len());
    println!("Make sure you have the same number of episode names as extracted files (or more)");
    println!("Aborting!!!");
  } else {
    let files_to_rename: Vec<_> =
      dirs
        .into_iter()
        .enumerate()
        .map(|(i, fne)|{
          let episode = episode_names.get(i).expect(&format!("could not read episode_names index: {}", i));
          let file_name_with_ext = format!("{}.{}",episode, fne.ext);
          let output_file_path = renames_directory.join(file_name_with_ext);
          let path_to_output_file = output_file_path.to_path_buf();
          Rename::new(fne.path, path_to_output_file)
        })
        .collect();


    println!("The following renames will be performed:");
    for f in &files_to_rename {
      println!("{:?} -> {:?}", f.from_file_name, f.to_file_name)
    }

    println!("");

    println!("Proceed with rename? 'y' to proceed or any other key to abort");
    let mut user_response = String::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    handle.read_line(&mut user_response).expect("Could not read from stdin"); // Unexpected, so throw
    let line = user_response.lines().next().expect("Could not extract line from buffer"); // Unexpected, so throw

    match line {
      "y" => perform_rename(&files_to_rename),
      _ => println!("aborting rename")
    }
  }
}

fn get_series_metadata(episode_guide: EpisodeGuide) -> SeriesMetaData {
  let metadata_regx = Regex::new(r"^.*/series/(?P<SERIES>.+)-(?P<TVDB>\d{5,})/season\-(?P<SEASON>\d+)\.json$").unwrap();

  let file_name = episode_guide.0.to_string_lossy().to_string();
  let captured =
    metadata_regx
      .captures(&file_name)
      .expect(&format!("Could not find captures in file path and name: {}.\nExpected file name config: <RENAMER_HOME>/series/SERIES_NAME-TVDBID/season-SEASON_NUMBER.json\nExample: /home/someone/.renamer/series/murdoch-mysteries-81670/season-1.json", file_name));

  let series = captured.name("SERIES").expect("Could not find series name in file name").as_str();
  let tvdb = captured.name("TVDB").expect("Could not find tvdb id  in file name").as_str();
  let season = captured.name("SEASON").expect("Could not find season number in file name").as_str();

  SeriesMetaData { name: series.to_owned(), tvdb_id: tvdb.to_owned(), season_number: season.to_owned() }
}

fn perform_rename(renames: &[Rename]) {
  for r in renames {
    fs::rename(&r.from_file_name, &r.to_file_name).expect(&format!("could not rename {:?} -> {:?}", &r.from_file_name, &r.to_file_name))
  }
}
