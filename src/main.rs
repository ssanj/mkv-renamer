use walkdir::WalkDir;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::fs;
use models::*;
use cli::*;
use std::error::Error;

mod models;
mod cli;
mod metadata_downloader;
mod html_scraper;

use html_scraper::get_series_metadata;
use metadata_downloader::download_metadata;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let config = get_cli_args();
  let processing_dir = config.processing_dir;
  let processing_dir_path = Path::new(&processing_dir);
  let processing_dir = ProcessingDir(processing_dir_path.to_path_buf());
  let metadata_input_type = config.metadata_input_type;

  let metadata_type = get_metadata_type(&metadata_input_type);

  match metadata_type {
    ConfigMetadataInputType::Url(url) =>
      handle_url_metadata(&url, &processing_dir).await?,
    ConfigMetadataInputType::File(file) => {
      let file_path = Path::new(&file);
      handle_file_metadata(&file_path, &processing_dir)
    }
  }

  Ok(())
}

async fn handle_url_metadata(url: &str, processing_dir: &ProcessingDir) -> Result<(), Box<dyn std::error::Error>> {
  let page_content = download_metadata(url).await?;
  let episodes_definition = get_series_metadata(&page_content);

  let processing_dir_path = processing_dir.0.as_path(); // TODO: Do not exposed internals
  if !processing_dir_path.exists() { // TODO: Handle processing validation in a common place
      println!("Processing directory does not exist:");
      print_error_if_file_not_found("processing_dir", processing_dir_path);
  } else {
    program(&processing_dir, &episodes_definition)
  }

  Ok(())
}

fn handle_file_metadata(series_metadata_path: &Path, processing_dir: &ProcessingDir) {
  let processing_dir_path = processing_dir.0.as_path(); // TODO: Do not exposed internals
  if !(series_metadata_path.exists() && processing_dir_path.exists()) { // TODO: Handle processing validation in a common place
      println!("One or more supplied file paths do not exist:");
      print_error_if_file_not_found("series_metadata", series_metadata_path);
      print_error_if_file_not_found("processing_dir", processing_dir_path);
  } else {
    let episodes_definition = read_episodes_from_file(series_metadata_path).expect("Could not load episode definitions");
    program(&processing_dir, &episodes_definition)
  }
}

enum ConfigMetadataInputType {
  Url(String),
  File(String)
}

fn get_metadata_type(input_type: &MetadataInputType) -> ConfigMetadataInputType {
  match (input_type.clone().url_metadata, input_type.clone().file_metadata) {
    (Some(url), _) => ConfigMetadataInputType::Url(url),
    (_, Some(file)) => ConfigMetadataInputType::File(file),
    _ => panic!("Invalid metadata configuration: {:?}", input_type)
  }
}

fn print_error_if_file_not_found(name: &str, p: &Path) {
  if !p.exists() {
    println!(" - Path for {} does not exist: {:?}", name, p)
  }
}

fn program(processing_dir: &ProcessingDir, episodes_definition: &EpisodesDefinition) {
  let metadata_episodes = &episodes_definition.episodes;
  let series_metadata = &episodes_definition.metadata;
  let rips_directory = processing_dir.rips_dir() ;
  let renames_directory = processing_dir.renames_dir();
  let encodes_directory = processing_dir.encodes_dir();

  let mut ripped_episode_filenames = get_ripped_episode_filenames(&rips_directory);
  // Sort disk file names in ascending order
  ripped_episode_filenames.sort_by(|fne1, fne2| fne1.partial_cmp(&fne2).unwrap());


  // We have more ripped episodes than metadata episode names. Abort.
  if ripped_episode_filenames.len() > metadata_episodes.len() {
    println!("Not enough metadata episode names ({}) to match ripped files ({})", metadata_episodes.len(), ripped_episode_filenames.len());
    println!("Make sure you have the same number of metadata episode names as ripped files (or more)");
    println!("Aborting!!!");
    std::process::exit(1)
  } else {
    let encoded_series_directory = get_series_directory(&encodes_directory, series_metadata);
    let encoded_series_directory_path = encoded_series_directory.as_path();

    let files_to_rename = get_files_to_rename(&ripped_episode_filenames, metadata_episodes, &renames_directory);

    let renames_result = confirm_changes(&files_to_rename, &encoded_series_directory_path);
    handle_renames_result(&renames_result, &files_to_rename);
    create_series_season_directories(encoded_series_directory_path);
  }
}

fn get_ripped_episode_filenames(rips_dir: &RipsDir) -> Vec<FileNameAndExt> {
  WalkDir::new(rips_dir)
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

fn get_files_to_rename(ripped_episode_filenames: &Vec<FileNameAndExt>, metadata_episodes: &Vec<EpisodeDefinition>, renames_dir: &RenamesDir) -> Vec<Rename> {
  let renames_dir_path = renames_dir.0.as_path(); // TODO: Don't expose internals

  ripped_episode_filenames
    .into_iter()
    .enumerate()
    .map(|(i, fne)|{
      let episode = metadata_episodes.get(i).expect(&format!("could not read metadata_episodes index: {}", i));
      let file_name_with_ext = format!("{} - {}.{}", episode.number, episode.name, fne.ext);

      let output_file_path = renames_dir_path.join(file_name_with_ext).to_path_buf();
      let path_to_output_file = output_file_path.to_path_buf();
      Rename::new(fne.clone().path, path_to_output_file)
    })
    .collect()
}

fn confirm_changes(files_to_rename: &Vec<Rename>, encodes_series_folder_structure: &Path) -> RenamesResult {
  println!("The following renames will be performed:");

  for f in files_to_rename {
    println!("{:?} -> {:?}", f.from_file_name, f.to_file_name)
  }
  println!("");

  println!("The following directory will be created:");
  println!("{}", encodes_series_folder_structure.to_string_lossy().to_string());
  println!("");

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

fn handle_renames_result(rename_result: &RenamesResult, files_to_rename: &Vec<Rename>) {
  match rename_result {
    RenamesResult::Correct => perform_rename(&files_to_rename),
    RenamesResult::Wrong => {
      println!("Aborting rename");
      std::process::exit(1)
    }
  }
}

fn create_series_season_directories(encoded_series_directory_path: &Path) {
  create_all_directories(encoded_series_directory_path).expect(&format!("Could not create encoded series directory: {}", encoded_series_directory_path.to_string_lossy()));
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
  let series_folder_structure = get_series_folder_structure(&series_metadata);
  encodes_dir.0.join(series_folder_structure) //TODO: Fix - we should expose the PathBuf internals
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
        "metadata": {
          "name":"Thundercats",
          "tvdb_id":"70355",
          "season_number":"1"
        },
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
          metadata: SeriesMetaData {
            name: "Thundercats".to_string(),
            tvdb_id: "70355".to_string(),
            season_number: "1".to_string()
          },
          episodes: expected_episodes
        };

      let episodes_definition: EpisodesDefinition = serde_json::from_str(conf).unwrap();
      assert_eq!(episodes_definition, expected_episodes_definition)
    }
}
