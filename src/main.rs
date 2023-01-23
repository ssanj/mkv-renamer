use walkdir::WalkDir;


use std::io::BufRead;
use std::path::Path;
use std::fs;
use models::*;

mod models;

fn main() {

  // TODO: Pass this in
  let working_dir = "/Volumes/MediaDrive/TV_Rips"; //current dir

  // TODO: Pass this in
  let target_dir = "/Volumes/MediaDrive/TV";

  // TODO: Pass this in via config file or read it from TVDB
  let episode_names =
    vec![
      Episode::new("S01E01", "Exodus", "70355"),
      Episode::new("S01E02", "The Unholy Alliance", "70355"),
      Episode::new("S01E03", "Berbils", "70355"),
      Episode::new("S01E04", "The Slaves of Castle Plun-Darr", "70355"),
      Episode::new("S01E05", "Pumm-Ra", "70355"),
      Episode::new("S01E06", "The Terror of Hammerhand", "70355"),
      Episode::new("S01E07", "Trouble with Time", "70355"),
      Episode::new("S01E08", "The Tower of Traps", "70355"),
      Episode::new("S01E09", "The Garden of Delights", "70355"),
      Episode::new("S01E10", "Mandora: The Evil Chaser", "70355"),
      Episode::new("S01E11", "The Ghost Warrior", "70355"),
      Episode::new("S01E12", "The Doomgaze", "70355"),
      Episode::new("S01E13", "Lord of the Snows", "70355"),
      Episode::new("S01E14", "The Spaceship Beneath the Sands", "70355"),
      Episode::new("S01E15", "The Time Capsule", "70355"),
      Episode::new("S01E16", "The Fireballs of Plun-Darr", "70355"),
      Episode::new("S01E17", "All that Glitters", "70355"),
      Episode::new("S01E18", "Spitting Image", "70355"),
      Episode::new("S01E19", "Mongor", "70355"),
      Episode::new("S01E20", "Return to Thundera", "70355"),
      Episode::new("S01E21", "Dr. Dometone", "70355"),
      Episode::new("S01E22", "The Astral Prison", "70355"),
      Episode::new("S01E23", "The Crystal Queen", "70355"),
      Episode::new("S01E24", "Safari Joe  Snarf Takes up the Challenge", "70355"),
      Episode::new("S01E26", "Sixth Sense", "70355"),
      Episode::new("S01E27", "The Thunder-Cutter", "70355"),
      Episode::new("S01E28", "The Wolfrat", "70355"),
      Episode::new("S01E29", "Feliner (1)", "70355"),
      Episode::new("S01E30", "Feliner (2)", "70355"),
      Episode::new("S01E31", "Mandora and the Pirates", "70355"),
      Episode::new("S01E32", "Return of the Driller", "70355"),
      Episode::new("S01E33", "Dimension Doom", "70355"),
      Episode::new("S01E34", "Queen of 8 Legs", "70355"),
      Episode::new("S01E35", "Sword in a Hole", "70355"),
      Episode::new("S01E36", "The Evil Harp of Charr-Nin", "70355"),
      Episode::new("S01E37", "Lion-O's Anointment First Day: The Trial of Strength", "70355"),
      Episode::new("S01E38", "The Demolisher", "70355"),
      Episode::new("S01E39", "Monkian's Bargain", "70355"),
      Episode::new("S01E40", "Tight Squeeze", "70355"),
      Episode::new("S01E41", "The Micrits", "70355"),
      Episode::new("S01E42", "Lion-O's Anointment Second Day: The Trial of Speed", "70355"),
      Episode::new("S01E43", "The Rock Giant", "70355"),
      Episode::new("S01E44", "Jackalman's Rebellion", "70355"),
      Episode::new("S01E45", "Turmagar the Tuska", "70355"),
      Episode::new("S01E46", "Lion-O's Anointment Third Day: The Trial of Cunning", "70355"),
      Episode::new("S01E47", "The Mumm-Ra Berbil", "70355"),
      Episode::new("S01E48", "Mechanical Plague", "70355"),
      Episode::new("S01E49", "Trapped", "70355"),
      Episode::new("S01E50", "Lion-O's Anointment Fourth Day: The Trial of Mind Power", "70355"),
      Episode::new("S01E51", "Excalibur", "70355"),
      Episode::new("S01E52", "Secret of the Ice King", "70355"),
      Episode::new("S01E53", "Good and Ugly", "70355"),
      Episode::new("S01E54", "The Transfer", "70355"),
      Episode::new("S01E55", "Divide and Conquer", "70355"),
      Episode::new("S01E56", "Dream Master", "70355"),
      Episode::new("S01E57", "Out of Sight", "70355"),
      Episode::new("S01E58", "The Mountain", "70355"),
      Episode::new("S01E59", "The Superpower Potion", "70355"),
      Episode::new("S01E60", "Eye of the Beholder", "70355"),
      Episode::new("S01E61", "Lion-O's Anointment Final Day: The Trial of Evil", "70355"),
      Episode::new("S01E62", "The Trouble with ThunderKittens", "70355"),
      Episode::new("S01E63",  "Mumm-Rana",  "70355"),
      Episode::new("S01E64",  "The Shifter",  "70355"),
      Episode::new("S01E65",  "Fond Memories",  "70355"),
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
