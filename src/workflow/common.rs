use serde::Deserialize;
use walkdir::WalkDir;
use std::io::{BufRead, BufReader, Write};
use console::Style;
use std::path::Path;
use std::fs;
use crate::models::*;
use crate::cli::*;

pub const ENCODES_FILE: &str = "encode_dir.txt";

pub fn get_metadata_type(input_type: &MetadataInputType) -> ConfigMetadataInputType {
  match (input_type.clone().url_metadata, input_type.clone().file_metadata) {
    (Some(url), _) => ConfigMetadataInputType::Url(url),
    (_, Some(file)) => ConfigMetadataInputType::File(file),
    _ => ConfigMetadataInputType::Invalid
  }
}


pub fn write_encodes_file<P: AsRef<Path>>(rename_dir: &RipsSessionRenamesDir, encoded_series_directory_path: P) -> R {
  let encodes_file = rename_dir.as_ref().join(ENCODES_FILE);
  let encodes_file_path = encodes_file.as_path();

  // Try to remove the old if it exists
  let _ = std::fs::remove_file(encodes_file_path);

  std::fs::OpenOptions::new()
    .create_new(true)
    .write(true)
    .open(encodes_file_path)
    .map_err(|e| RenamerError::CouldNotOpenEncodesFile(encodes_file.clone(), e.to_string()))
    .and_then(|mut file| {
      file.write(encoded_series_directory_path.as_ref().to_string_lossy().as_bytes())
        .and(file.flush())
        .map_err(|e| RenamerError::CouldNotWriteEncodesFile(encodes_file, e.to_string()))
    })
    .map(|_| ())
}

pub fn get_ripped_filenames(rips_session_number: &RipsSessionNumberDir) -> Vec<FileNameAndExt> {
  WalkDir::new(rips_session_number)
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
    .collect()
}



pub fn confirm_changes(files_to_rename: &Vec<Rename>, encodes_series_folder_structure: &Path) -> RenamesResult {
  println!("The following renames will be performed:");
  let yellow = Style::new().yellow();

  for f in files_to_rename {
    println!("{:?} -> {:?}", f.from_file_name, yellow.apply_to(f.to_file_name.as_path().to_string_lossy()))
  }
  println!();

  println!("The following directory will be created:");
  println!("{}", yellow.apply_to(encodes_series_folder_structure.to_string_lossy().to_string()));
  println!();

  println!("Proceed? 'y' to proceed or any other key to abort");

  let mut user_response = String::new();
  let stdin = std::io::stdin();
  let mut handle = stdin.lock();
  handle.read_line(&mut user_response).expect("Could not read from stdin"); // Unexpected, so throw
  let line = user_response.lines().next().expect("Could not extract line from buffer"); // Unexpected, so throw

  match line {
    "y" => RenamesResult::Correct,
    _ => RenamesResult::Wrong
  }
}


pub fn create_series_season_directories(encoded_series_directory_path: &Path) -> R {
  create_all_directories(encoded_series_directory_path)
}

// Fails if the directory already exists
pub fn create_all_directories(p: &Path) -> R {
  // We want to fail if the directory already exists
  if !p.exists() {
    fs::create_dir_all(p)
      .map_err(|e| {
        RenamerError::CouldNotCreatedSeriesDirectory(<Path as AsRef<Path>>::as_ref(p).to_owned(), e.to_string())
      })
  } else {
    Err(RenamerError::SeriesDirectoryAlreadyExists(p.to_owned()))
  }
}


pub fn perform_rename(renames: &[Rename]) {
  for r in renames {
    fs::rename(&r.from_file_name, &r.to_file_name).unwrap_or_else(|e| panic!("could not rename {:?} -> {:?}, due to: {}", &r.from_file_name, &r.to_file_name, e))
  }
}

pub fn read_input_from_file<P: AsRef<Path>, R: for<'a> Deserialize<'a>>(path: P) -> Result<R, RenamerError> {
  let file =
    fs::File::open(&path)
      .map_err(|e| RenamerError::CouldNotAccessMetadataFile(path.as_ref().to_string_lossy().to_string(), e.to_string()))?;

  let reader = BufReader::new(file);
  let u =
    serde_json::from_reader(reader)
      .map_err(|e| RenamerError::CouldNotDecodeMetadataFileJson(path.as_ref().to_owned(), e.to_string()))?;

  Ok(u)
}
