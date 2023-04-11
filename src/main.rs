use walkdir::WalkDir;


use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::fs;
use models::*;
use cli::*;
use regex::Regex;
use std::error::Error;

mod models;
mod cli;


//cargo run -- -s /Users/sanj/ziptemp/renaming/series/murdoch-mysteries-81670/season-5.json -d /Volumes/MediaDrive/TV_Rips -r /Volumes/MediaDrive/TV
fn main() {
  let config = get_cli_args();
  let series_metadata_file = config.series_metadata;
  let dvd_rips_directory = config.dvd_rips;
  let renames_directory = config.renames_directory;

  let series_metadata_path = Path::new(&series_metadata_file);
  let dvd_rips_directory_path = Path::new(&dvd_rips_directory);
  let renames_directory_path = Path::new(&renames_directory);

  if !(series_metadata_path.exists() && dvd_rips_directory_path.exists() && renames_directory_path.exists()) {
      println!("One or more supplied file paths do not exist:");
      print_error_if_file_not_found("series_metadata", series_metadata_path);
      print_error_if_file_not_found("dvd_rips_directory", dvd_rips_directory_path);
      print_error_if_file_not_found("renames_directory", renames_directory_path)
  } else {
    let series_metadata = get_series_metadata(EpisodeGuide(series_metadata_path.to_owned()));

    let episodes_definition = read_episodes_from_file(series_metadata_path).expect("Could not load episode definitions");
    let episodes = definitions_to_episodes(episodes_definition, &series_metadata);

    let dvd_rips_directory = DvdRipsDir(dvd_rips_directory_path.to_path_buf());
    let renames_directory = RenamesDir(renames_directory_path.to_path_buf());

    program(&series_metadata, &dvd_rips_directory, &renames_directory, &episodes)
  }
}

fn definitions_to_episodes(episodes_definition: EpisodesDefinition, series_metadata: &SeriesMetaData) -> Vec<Episode>{
  episodes_definition
    .episodes
    .into_iter()
    .map(|ed| {
      Episode::new(&ed.number, &ed.name, &series_metadata.tvdb_id)
    })
    .collect()
}


fn print_error_if_file_not_found(name: &str, p: &Path) {
  if !p.exists() {
    println!(" - Path for {} does not exist: {:?}", name, p)
  }
}

fn program(series_metadata: &SeriesMetaData, dvd_rips: &DvdRipsDir, renames_dir: &RenamesDir, episodes: &Vec<Episode>) {

  let dvd_rips_directory =  &dvd_rips.0; //"/Volumes/MediaDrive/TV_Rips"; //current dir
  let renames_directory = &renames_dir.0;//"/Volumes/MediaDrive/TV";

  let mut dirs: Vec<FileNameAndExt> = WalkDir::new(dvd_rips_directory)
      .into_iter()
      .filter_map(|re| re.ok())
      .filter_map(|dir_entry| {
        let p = dir_entry.path();
        let is_file = p.is_file();
        let has_disk_subdirectory = p.to_string_lossy().to_string().contains("/disc");
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

  if dirs.len() > episodes.len() {
    println!("Not enough Episode names ({}) to match actual files extracted ({})", episodes.len(), dirs.len());
    println!("Make sure you have the same number of episode names as extracted files (or more)");
    println!("Aborting!!!");
  } else {
    let series_directory = get_series_directory(renames_directory, series_metadata);
    let series_directory_path = series_directory.as_path();
    let files_to_rename: Vec<_> =
      dirs
        .into_iter()
        .enumerate()
        .map(|(i, fne)|{
          let episode = episodes.get(i).expect(&format!("could not read episodes index: {}", i));
          let file_name_with_ext = format!("{}.{}",episode, fne.ext);
          let output_file_path = series_directory_path.join(file_name_with_ext).to_path_buf();
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
      "y" => {
        create_all_directories(series_directory_path).expect(&format!("Could not create series directory: {}", series_directory_path.to_string_lossy()));
        perform_rename(&files_to_rename)
      },
      _ => println!("aborting rename")
    }
  }
}

// Fails if the directory already exists
fn create_all_directories(p: &Path) -> std::io::Result<()> {
  // We want to fail if the directory already exists
  if !p.exists() {
    fs::create_dir_all(p)
  } else {
    Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Series directory already exists: {}. Aborting", p.to_string_lossy())))
  }
}

fn get_series_directory(renames_directory: &PathBuf, series_metadata: &SeriesMetaData) -> PathBuf {
  use convert_case::{Case, Casing};
  let series_name =  series_metadata.name.to_case(Case::Title);
  let tvdb_id = &series_metadata.tvdb_id;
  let season = format!("{}", series_metadata.season_number);
  let parent_dirs = format!("{} {{tvdb-{}}}/Season {:0>2}", series_name, tvdb_id, season);
  renames_directory.as_path().join(parent_dirs).to_path_buf()
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

fn read_episodes_from_file<P: AsRef<Path>>(path: P) -> Result<EpisodesDefinition, Box<dyn Error>> {
  let file = fs::File::open(path)?;
  let reader = BufReader::new(file);
  let u = serde_json::from_reader(reader)?;
  Ok(u)
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_episode_deserialization() {
      let conf = r#"{
        "episodes": [
          { "number":"S01E01", "name":"Exodus"},
          { "number":"S01E02", "name":"The Unholy Alliance"},
          { "number":"S01E03", "name":"Berbils"},
          { "number":"S01E04", "name":"The Slaves of Castle Plun-Darr"},
          { "number":"S01E05", "name":"Pumm-Ra"},
          { "number":"S01E06", "name":"The Terror of Hammerhand"}
        ]
      }"#;

      let expected_episodes =
        vec![
          EpisodeDefinition {
            number:"S01E01".to_string(),
            name:"Exodus".to_string()
          },
          EpisodeDefinition {
            number:"S01E02".to_string(),
            name:"The Unholy Alliance".to_string()
          },
          EpisodeDefinition {
            number:"S01E03".to_string(),
            name:"Berbils".to_string()
          },
          EpisodeDefinition {
            number:"S01E04".to_string(),
            name:"The Slaves of Castle Plun-Darr".to_string()
          },
          EpisodeDefinition {
            number:"S01E05".to_string(),
            name:"Pumm-Ra".to_string()
          },
          EpisodeDefinition {
            number:"S01E06".to_string(),
            name:"The Terror of Hammerhand".to_string()
          }
        ];

      let expected_episodes_definition =
        EpisodesDefinition {
          episodes: expected_episodes
        };

      let episodes_definition: EpisodesDefinition = serde_json::from_str(conf).unwrap();
      assert_eq!(episodes_definition, expected_episodes_definition)
    }
}
