use walkdir::WalkDir;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::fs;
use std::error::Error;
use console::{style, Style};

use crate::html_scraper::get_series_metadata;
use crate::metadata_downloader::download_metadata;
use crate::models::*;
use crate::cli::*;


pub async fn perform_workflow(config: MkvRenamerArgs) -> Result<(), Box<dyn Error>> {
  let processing_dir_path = Path::new(&config.processing_dir);
  let processing_dir = ProcessingDir(processing_dir_path.to_path_buf());
  let session_dir = SessionDir::new(config.session_dir);
  let metadata_input_type = config.metadata_input_type;

  let metadata_type = get_metadata_type(&metadata_input_type);

  match metadata_type {
    ConfigMetadataInputType::Url(url) =>
      handle_url_metadata(&url, &processing_dir, &session_dir, config.verbose).await?,
    ConfigMetadataInputType::File(file) => {
      let file_path = Path::new(&file);
      handle_file_metadata(file_path, &processing_dir, &session_dir, config.verbose)
    },
    ConfigMetadataInputType::Invalid => eprintln!("{}", style(format!("Invalid metadata configuration: {:?}", metadata_input_type)).red())
  }

  Ok(())
}

async fn handle_url_metadata(url: &str, processing_dir: &ProcessingDir, session_dir: &SessionDir, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
  let page_content = download_metadata(url).await?;
  let episodes_definition = get_series_metadata(&page_content);

  let processing_dir_path = processing_dir.as_ref();
  if !processing_dir_path.exists() { // TODO: Handle processing validation in a common place
      eprintln!("{}", style("Processing directory does not exist:").red());
      print_error_if_file_not_found("processing_dir", processing_dir_path);
  } else {
    program(processing_dir, session_dir, &episodes_definition, verbose)
  }

  Ok(())
}

fn handle_file_metadata(series_metadata_path: &Path, processing_dir: &ProcessingDir, session_dir: &SessionDir, verbose: bool) {
  let processing_dir_path = processing_dir.as_ref();
  if !(series_metadata_path.exists() && processing_dir_path.exists()) { // TODO: Handle processing validation in a common place
      eprintln!("{}", style("One or more supplied file paths do not exist:").red());
      print_error_if_file_not_found("series_metadata", series_metadata_path);
      print_error_if_file_not_found("processing_dir", processing_dir_path);
  } else {
    let episodes_definition = read_episodes_from_file(series_metadata_path).expect("Could not load episode definitions");
    program(processing_dir, session_dir, &episodes_definition, verbose)
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

fn print_error_if_file_not_found(name: &str, p: &Path) {
  if !p.exists() {
    eprintln!("- {}", style(format!("Path for {} does not exist: {:?}", name, p)).yellow())
  }
}

fn program(processing_dir: &ProcessingDir, session_dir: &SessionDir, episodes_definition: &EpisodesDefinition, verbose: bool) {
  let metadata_episodes = &episodes_definition.episodes;
  let series_metadata = &episodes_definition.metadata;

  let rips_directory = processing_dir.rips_session_dir(session_dir);
  let renames_directory = processing_dir.rips_session_renames_dir(session_dir);
  let encodes_directory = processing_dir.encodes_dir();

  if verbose {
    let cyan = Style::new().cyan();
    println!();
    println!("{}: {} (Root directory)",  cyan.apply_to("processing dir"), processing_dir.as_ref().to_string_lossy());
    println!("{}: {} (Contains disc1..N with .mkv files)", cyan.apply_to("session dir"), processing_dir.rips_session_dir(session_dir).as_ref().to_string_lossy());
    println!("{}: {} (Stores renamed episodes)", cyan.apply_to("rename dir"), processing_dir.rips_session_renames_dir(session_dir).as_ref().to_string_lossy());
    println!("{}: {} (Stores encoded episodes)", cyan.apply_to("encode dir"), processing_dir.encodes_dir().as_ref().to_string_lossy());
    println!()
  }

  let mut ripped_episode_filenames = get_ripped_episode_filenames(&rips_directory);
  // Sort disk file names in ascending order
  ripped_episode_filenames.sort_by(|fne1, fne2| fne1.partial_cmp(fne2).unwrap());


  // We have more ripped episodes than metadata episode names. Abort.
  if ripped_episode_filenames.len() > metadata_episodes.len() {
    let red = Style::new().red();
    eprintln!("{}", red.apply_to(format!("Not enough metadata episode names ({}) to match ripped files ({})", metadata_episodes.len(), ripped_episode_filenames.len())));
    eprintln!("{}", red.apply_to("Make sure you have the same number of metadata episode names as ripped files (or more)"));
    eprintln!("{}", red.apply_to("Aborting!!!"));
    std::process::exit(1)
  } else {
    let encoded_series_directory = get_series_directory(&encodes_directory, series_metadata);
    let encoded_series_directory_path = encoded_series_directory.as_path();

    let files_to_rename = get_files_to_rename(&ripped_episode_filenames, metadata_episodes, &renames_directory);

    if !files_to_rename.is_empty() {
      let renames_result = confirm_changes(&files_to_rename, encoded_series_directory_path);
      handle_renames_result(&renames_result, &files_to_rename);
      create_series_season_directories(encoded_series_directory_path);
    } else {
      eprintln!("{}", style("No files found to rename").red())
    }
  }
}

fn get_ripped_episode_filenames(rips_session_dir: &RipsSessionDir) -> Vec<FileNameAndExt> {
  WalkDir::new(rips_session_dir)
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

fn handle_renames_result(rename_result: &RenamesResult, files_to_rename: &[Rename]) {
  match rename_result {
    RenamesResult::Correct => perform_rename(files_to_rename),
    RenamesResult::Wrong => {
      println!("Aborting rename");
      std::process::exit(1)
    }
  }
}

fn create_series_season_directories(encoded_series_directory_path: &Path) {
  create_all_directories(encoded_series_directory_path).unwrap_or_else(|e| panic!("Could not create encoded series directory: {}, due to: {}", encoded_series_directory_path.to_string_lossy(), e));
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

fn get_series_directory(encodes_dir: &EncodesDir, series_metadata: &SeriesMetaData) -> PathBuf {
  let series_folder_structure = get_series_folder_structure(series_metadata);
  encodes_dir.join(series_folder_structure)
}


fn perform_rename(renames: &[Rename]) {
  for r in renames {
    fs::rename(&r.from_file_name, &r.to_file_name).unwrap_or_else(|e| panic!("could not rename {:?} -> {:?}, due to: {}", &r.from_file_name, &r.to_file_name, e))
  }
}

fn read_episodes_from_file<P: AsRef<Path>>(path: P) -> Result<EpisodesDefinition, Box<dyn Error>> {
  let file = fs::File::open(path)?;
  let reader = BufReader::new(file);
  let u = serde_json::from_reader(reader)?;
  Ok(u)
}
