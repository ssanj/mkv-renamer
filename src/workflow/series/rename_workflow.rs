use std::path::{Path, PathBuf};
use crate::html_scraper::get_series_metadata;
use crate::metadata_downloader::download_metadata;
use crate::models::*;
use crate::cli::*;
use super::super::common;

pub async fn perform(rename_args: RenameArgs) -> ROutput {
  let processing_dir_path = Path::new(&rename_args.processing_dir);
  let processing_dir = ProcessingDir(processing_dir_path.to_path_buf());
  let session_number = SessionNumberDir::new(rename_args.session_number);
  let metadata_input_type = rename_args.metadata_input_type;

  let metadata_type = common::get_metadata_type(&metadata_input_type);

  match metadata_type {
    ConfigMetadataInputType::Url(url) =>
      handle_url_metadata(&url, &processing_dir, &session_number, rename_args.verbose, rename_args.skip_files).await,
    ConfigMetadataInputType::File(file) => {
      let file_path = Path::new(&file);
      handle_file_metadata(file_path, &processing_dir, &session_number, rename_args.verbose, rename_args.skip_files)
    },
    ConfigMetadataInputType::Invalid => Err(RenamerError::InvalidMetadataConfiguration(format!("{:?}", &metadata_input_type))),
  }
}


async fn handle_url_metadata(url: &str, processing_dir: &ProcessingDir, session_number: &SessionNumberDir, verbose: bool, skip_files: bool) -> ROutput {
  let page_content = download_metadata(url).await?;
  let episodes_definition = get_series_metadata(&page_content);

  let processing_dir_path = processing_dir.as_ref();
  if !processing_dir_path.exists() {
      Err(RenamerError::ProcessingDirectoryDoesNotExist(processing_dir_path.to_owned()))
  } else {
    program(processing_dir, session_number, &episodes_definition, verbose, skip_files)
  }
}


fn handle_file_metadata(series_metadata_path: &Path, processing_dir: &ProcessingDir, session_number: &SessionNumberDir, verbose: bool, skip_files: bool) -> ROutput {
  let processing_dir_path = processing_dir.as_ref();
  match (series_metadata_path.exists(), processing_dir_path.exists()) {
      (true, true) => {
        let episodes_definition = common::read_input_from_file(series_metadata_path)?;
        program(processing_dir, session_number, &episodes_definition, verbose, skip_files)
      },
      (false, false) => Err(RenamerError::ProcessingDirAndMetadaPathDoesNotExit(processing_dir_path.to_owned(), series_metadata_path.to_owned())),
      (_, false) => Err(RenamerError::ProcessingDirectoryDoesNotExist(processing_dir_path.to_owned())),
      (false, _) => Err(RenamerError::MetadataDirectoryDoesNotExist(series_metadata_path.to_owned())),
  }
}


fn program(processing_dir: &ProcessingDir, session_number: &SessionNumberDir, episodes_definition: &EpisodesDefinition, verbose: bool, skip_files: bool) -> ROutput {
  let metadata_episodes = &episodes_definition.episodes;
  let series_metadata = &episodes_definition.metadata;

  let rips_directory = processing_dir.rips_session_number(session_number);
  let renames_directory = processing_dir.rips_session_renames_dir(session_number);
  let encodes_directory = processing_dir.tv_encodes_dir();

  common::dump_processing_info(processing_dir, session_number, verbose);

  // We want to skip files.
  // Only create output directory and encodes file.
  if skip_files {
      let encoded_series_directory = get_series_directory(&encodes_directory, series_metadata);
      let encoded_series_directory_path = encoded_series_directory.as_path();

      if encoded_series_directory_path.exists() {
        return Err(RenamerError::SeriesDirectoryAlreadyExists(encoded_series_directory))
      }

    common::create_all_directories(encoded_series_directory_path)
      .and(common::write_encodes_file(&renames_directory, encoded_series_directory_path))
      .map(|_| Output::Success)
  } else {
    let mut ripped_episode_filenames = common::get_ripped_filenames(&rips_directory);
    // Sort disk file names in ascending order
    ripped_episode_filenames.sort_by(|fne1, fne2| fne1.partial_cmp(fne2).unwrap());

    // We have more ripped episodes than metadata episode names. Abort.
    if ripped_episode_filenames.len() > metadata_episodes.len() {
      Err(RenamerError::NotEnoughMetadataForEpisodes(metadata_episodes.len(), ripped_episode_filenames.len()))
    } else {
      let encoded_series_directory = get_series_directory(&encodes_directory, series_metadata);
      let encoded_series_directory_path = encoded_series_directory.as_path();

      if encoded_series_directory_path.exists() {
        return Err(RenamerError::SeriesDirectoryAlreadyExists(encoded_series_directory))
      }

      let files_to_rename = get_files_to_rename(&ripped_episode_filenames, metadata_episodes, &renames_directory);

      if !files_to_rename.is_empty() {
        match common::confirm_changes(&files_to_rename, encoded_series_directory_path) {
          RenamesResult::Correct => {
            common::perform_rename(&files_to_rename);
            common::create_all_directories(encoded_series_directory_path)
              .and(common::write_encodes_file(&renames_directory, encoded_series_directory_path))
              .map(|_| Output::Success)
          },
          RenamesResult::Wrong => Ok(Output::UserCanceled)
        }
      } else {
        Err(RenamerError::NoFilesToRename)
      }
    }
  }
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


fn get_series_folder_structure(series_metadata: &SeriesMetaData) -> String {
  let series_name = series_metadata.name.clone();
  let tvdb_id = series_metadata.tvdb_id.clone();
  let season_number = series_metadata.season_number.clone();
  format!("{} {{tvdb-{}}}/Season {:0>2}", series_name, tvdb_id, season_number)
}


fn get_series_directory(encodes_dir: &EncodesDir, series_metadata: &SeriesMetaData) -> PathBuf {
  let series_folder_structure = get_series_folder_structure(series_metadata);
  encodes_dir.join(series_folder_structure)
}
