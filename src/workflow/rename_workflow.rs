use walkdir::WalkDir;
use std::io::{BufRead, BufReader, Write};
use console::Style;
use std::path::{Path, PathBuf};
use std::fs;
use crate::html_scraper::get_series_metadata;
use crate::metadata_downloader::download_metadata;
use crate::models::*;
use crate::cli::*;

pub const ENCODES_FILE: &str = "encode_dir.txt";

pub async fn perform(rename_args: RenameArgs) -> ROutput {
  let processing_dir_path = Path::new(&rename_args.processing_dir);
  let processing_dir = ProcessingDir(processing_dir_path.to_path_buf());
  let session_number = SessionNumberDir::new(rename_args.session_number);
  let metadata_input_type = rename_args.metadata_input_type;

  let metadata_type = get_metadata_type(&metadata_input_type);

  match metadata_type {
    ConfigMetadataInputType::Url(url) =>
      handle_url_metadata(&url, &processing_dir, &session_number, rename_args.verbose).await,
    ConfigMetadataInputType::File(file) => {
      let file_path = Path::new(&file);
      handle_file_metadata(file_path, &processing_dir, &session_number, rename_args.verbose)
    },
    ConfigMetadataInputType::Invalid => Err(RenamerError::InvalidMetadataConfiguration(format!("{:?}", &metadata_input_type))),
  }
}


async fn handle_url_metadata(url: &str, processing_dir: &ProcessingDir, session_number: &SessionNumberDir, verbose: bool) -> ROutput {
  let page_content = download_metadata(url).await?;
  let episodes_definition = get_series_metadata(&page_content);

  let processing_dir_path = processing_dir.as_ref();
  if !processing_dir_path.exists() {
      Err(RenamerError::ProcessingDirectoryDoesNotExist(processing_dir_path.to_owned()))
  } else {
    program(processing_dir, session_number, &episodes_definition, verbose)
  }
}

fn handle_file_metadata(series_metadata_path: &Path, processing_dir: &ProcessingDir, session_number: &SessionNumberDir, verbose: bool) -> ROutput {
  let processing_dir_path = processing_dir.as_ref();
  match (series_metadata_path.exists(), processing_dir_path.exists()) {
      (true, true) => {
        let episodes_definition = read_episodes_from_file(series_metadata_path)?;
        program(processing_dir, session_number, &episodes_definition, verbose)
      },
      (false, false) => Err(RenamerError::ProcessingDirAndMetadaPathDoesNotExit(processing_dir_path.to_owned(), series_metadata_path.to_owned())),
      (_, false) => Err(RenamerError::ProcessingDirectoryDoesNotExist(processing_dir_path.to_owned())),
      (false, _) => Err(RenamerError::MetadataDirectoryDoesNotExist(series_metadata_path.to_owned())),
  }
}

enum ConfigMetadataInputType {
  Url(String),
  File(String),
  Invalid
}

fn get_metadata_type(input_type: &MetadataInputType) -> ConfigMetadataInputType {
  match (input_type.clone().url_metadata, input_type.clone().file_metadata) {
    (Some(url), _) => ConfigMetadataInputType::Url(url),
    (_, Some(file)) => ConfigMetadataInputType::File(file),
    _ => ConfigMetadataInputType::Invalid
  }
}

fn program(processing_dir: &ProcessingDir, session_number: &SessionNumberDir, episodes_definition: &EpisodesDefinition, verbose: bool) -> ROutput {
  let metadata_episodes = &episodes_definition.episodes;
  let series_metadata = &episodes_definition.metadata;

  let rips_directory = processing_dir.rips_session_number(session_number);
  let renames_directory = processing_dir.rips_session_renames_dir(session_number);
  let encodes_directory = processing_dir.encodes_dir();

  if verbose {
    let cyan = Style::new().cyan();
    println!();
    println!("{}: {} (Root directory)",  cyan.apply_to("processing dir"), processing_dir.as_ref().to_string_lossy());
    println!("{}: {} (Contains disc1..N with .mkv files)", cyan.apply_to("session dir"), processing_dir.rips_session_number(session_number).as_ref().to_string_lossy());
    println!("{}: {} (Stores renamed episodes)", cyan.apply_to("rename dir"), processing_dir.rips_session_renames_dir(session_number).as_ref().to_string_lossy());
    println!("{}: {} (Stores encoded episodes)", cyan.apply_to("encode dir"), processing_dir.encodes_dir().as_ref().to_string_lossy());
    println!()
  }

  let mut ripped_episode_filenames = get_ripped_episode_filenames(&rips_directory);
  // Sort disk file names in ascending order
  ripped_episode_filenames.sort_by(|fne1, fne2| fne1.partial_cmp(fne2).unwrap());


  // We have more ripped episodes than metadata episode names. Abort.
  if ripped_episode_filenames.len() > metadata_episodes.len() {
    Err(RenamerError::NotEnoughMetadataForEpisodes(metadata_episodes.len(), ripped_episode_filenames.len()))
  } else {
    let encoded_series_directory = get_series_directory(&encodes_directory, series_metadata);
    let encoded_series_directory_path = encoded_series_directory.as_path();

    let files_to_rename = get_files_to_rename(&ripped_episode_filenames, metadata_episodes, &renames_directory);

    if !files_to_rename.is_empty() {
      match confirm_changes(&files_to_rename, encoded_series_directory_path) {
        RenamesResult::Correct => {
          perform_rename(&files_to_rename);
          create_series_season_directories(encoded_series_directory_path)
            .and(write_encodes_file(&renames_directory, encoded_series_directory_path))
            .map(|_| Output::Success)
        },
        RenamesResult::Wrong => Ok(Output::UserCanceled)
      }
    } else {
      Err(RenamerError::NoFilesToRename)
    }
  }
}

fn write_encodes_file<P: AsRef<Path>>(rename_dir: &RipsSessionRenamesDir, encoded_series_directory_path: P) -> R {
  let encodes_file = rename_dir.as_ref().join(ENCODES_FILE);
  let encodes_file_path = encodes_file.as_path();
  std::fs::OpenOptions::new()
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

fn get_ripped_episode_filenames(rips_session_number: &RipsSessionNumberDir) -> Vec<FileNameAndExt> {
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

fn get_files_to_rename(ripped_episode_filenames: &[FileNameAndExt], metadata_episodes: &[EpisodeDefinition], renames_dir: &RipsSessionRenamesDir) -> Vec<Rename> {
  let renames_dir_path = renames_dir.as_ref();

  ripped_episode_filenames
    .iter()
    .enumerate()
    .map(|(i, fne)|{
      let episode = metadata_episodes.get(i).unwrap_or_else(|| panic!("could not read metadata_episodes index: {}", i));
      let file_name_with_ext = format!("{} - {}.{}", episode.number, episode.name, fne.ext);

      let output_file_path = renames_dir_path.join(file_name_with_ext).to_path_buf();
      let path_to_output_file = output_file_path.to_path_buf();
      Rename::new(fne.clone().path, path_to_output_file)
    })
    .collect()
}

fn confirm_changes(files_to_rename: &Vec<Rename>, encodes_series_folder_structure: &Path) -> RenamesResult {
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

fn get_series_folder_structure(series_metadata: &SeriesMetaData) -> String {
  let series_name = series_metadata.name.clone();
  let tvdb_id = series_metadata.tvdb_id.clone();
  let season_number = series_metadata.season_number.clone();
  format!("{} {{tvdb-{}}}/Season {:0>2}", series_name, tvdb_id, season_number)
}


fn create_series_season_directories(encoded_series_directory_path: &Path) -> R {
  create_all_directories(encoded_series_directory_path)
}

// Fails if the directory already exists
fn create_all_directories(p: &Path) -> R {
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

fn get_series_directory(encodes_dir: &EncodesDir, series_metadata: &SeriesMetaData) -> PathBuf {
  let series_folder_structure = get_series_folder_structure(series_metadata);
  encodes_dir.join(series_folder_structure)
}


fn perform_rename(renames: &[Rename]) {
  for r in renames {
    fs::rename(&r.from_file_name, &r.to_file_name).unwrap_or_else(|e| panic!("could not rename {:?} -> {:?}, due to: {}", &r.from_file_name, &r.to_file_name, e))
  }
}

fn read_episodes_from_file<P: AsRef<Path>>(path: P) -> Result<EpisodesDefinition, RenamerError> {
  let file =
    fs::File::open(&path)
      .map_err(|e| RenamerError::CouldNotAccessMetadataFile(path.as_ref().to_string_lossy().to_string(), e.to_string()))?;

  let reader = BufReader::new(file);
  let u =
    serde_json::from_reader(reader)
      .map_err(|e| RenamerError::CouldNotDecodeEpisodeJson(path.as_ref().to_owned(), e.to_string()))?;

  Ok(u)
}
