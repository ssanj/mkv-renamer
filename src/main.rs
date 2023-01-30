use walkdir::WalkDir;


use std::io::BufRead;
use std::path::Path;
use std::fs;
use models::*;
use cli::*;

mod models;
mod cli;

fn main() {
  let config = get_cli_args();
  println!("config: {:?}", config)
}

fn program() {
  // TODO: Pass this in
  let working_dir = "/Volumes/MediaDrive/TV_Rips"; //current dir

  // TODO: Pass this in
  let target_dir = "/Volumes/MediaDrive/TV";

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

  let mut dirs: Vec<FileNameAndExt> = WalkDir::new(working_dir)
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
          let output_file_path = Path::new(target_dir).join(file_name_with_ext);
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

fn perform_rename(renames: &[Rename]) {
  for r in renames {
    fs::rename(&r.from_file_name, &r.to_file_name).expect(&format!("could not rename {:?} -> {:?}", &r.from_file_name, &r.to_file_name))
  }
}
