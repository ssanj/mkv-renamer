use std::path::{Path, PathBuf};
use crate::html_scraper::get_movie_definition;
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
  let movie_definition = get_movie_definition(&page_content);

  let processing_dir_path = processing_dir.as_ref();
  if !processing_dir_path.exists() {
      Err(RenamerError::ProcessingDirectoryDoesNotExist(processing_dir_path.to_owned()))
  } else {
    program(processing_dir, session_number, &movie_definition, verbose, skip_files)
  }
}

fn handle_file_metadata(series_metadata_path: &Path, processing_dir: &ProcessingDir, session_number: &SessionNumberDir, verbose: bool, skip_files: bool) -> ROutput {
  let processing_dir_path = processing_dir.as_ref();
  match (series_metadata_path.exists(), processing_dir_path.exists()) {
      (true, true) => {
        let movie_definition: MovieDefinition = common::read_input_from_file(series_metadata_path)?;
        program(processing_dir, session_number, &movie_definition, verbose, skip_files)
      },
      (false, false) => Err(RenamerError::ProcessingDirAndMetadaPathDoesNotExit(processing_dir_path.to_owned(), series_metadata_path.to_owned())),
      (_, false) => Err(RenamerError::ProcessingDirectoryDoesNotExist(processing_dir_path.to_owned())),
      (false, _) => Err(RenamerError::MetadataDirectoryDoesNotExist(series_metadata_path.to_owned())),
  }
}


fn program(processing_dir: &ProcessingDir, session_number: &SessionNumberDir, movie_definition: &MovieDefinition, verbose: bool, skip_files: bool) -> ROutput {
  let rips_directory = processing_dir.rips_session_number(session_number);
  let renames_directory = processing_dir.rips_session_renames_dir(session_number);
  let encodes_directory = processing_dir.movies_encodes_dir();

  common::dump_processing_info(processing_dir, session_number, verbose);

  let ripped_filenames = common::get_ripped_filenames(&rips_directory);

  // Skip files.
  // Only create encode file and output directory
  if skip_files {
    let encoded_movie_directory = get_movie_directory(&encodes_directory, movie_definition);
    let encoded_movie_directory_path = encoded_movie_directory.as_path();

    if encoded_movie_directory_path.exists() {
      return Err(RenamerError::MovieDirectoryAlreadyExists(encoded_movie_directory))
    }

    common::create_all_directories(encoded_movie_directory_path)
      .and(common::write_encodes_file(&renames_directory, encoded_movie_directory_path))
      .map(|_| Output::Success)
  } else if ripped_filenames.is_empty() {
    Err(RenamerError::NoMovieDefinitionFound)
  } else {
    let encoded_movie_directory = get_movie_directory(&encodes_directory, movie_definition);
    let encoded_movie_directory_path = encoded_movie_directory.as_path();

    if encoded_movie_directory_path.exists() {
      return Err(RenamerError::MovieDirectoryAlreadyExists(encoded_movie_directory))
    }

    let files_to_rename = get_files_to_rename(&ripped_filenames, movie_definition, &renames_directory);

    if !files_to_rename.is_empty() {
      match common::confirm_changes(&files_to_rename, encoded_movie_directory_path) {
        RenamesResult::Correct => {
          common::perform_rename(&files_to_rename);
          common::create_all_directories(encoded_movie_directory_path)
            .and(common::write_encodes_file(&renames_directory, encoded_movie_directory_path))
            .map(|_| Output::Success)
        },
        RenamesResult::Wrong => Ok(Output::UserCanceled)
      }
    } else {
      Err(RenamerError::NoFilesToRename)
    }
  }
}


fn get_files_to_rename(ripped_movie_names: &[FileNameAndExt], movie_definition: &MovieDefinition, renames_dir: &RipsSessionRenamesDir) -> Vec<Rename> {
  let renames_dir_path = renames_dir.as_ref();

  ripped_movie_names
    .iter()
    .map(|fne|{
      let movie_name = movie_definition.name();
      let tvdb_id = movie_definition.tvdb_id();
      let ext = &fne.ext;
      let file_name_with_ext = format!("{movie_name} - {{tvdb-{tvdb_id}}} [tvdbid-{tvdb_id}].{ext}");
      let output_file_path = renames_dir_path.join(file_name_with_ext).to_path_buf();
      let path_to_output_file = output_file_path.to_path_buf();
      Rename::new(fne.clone().path, path_to_output_file)
    })
    .collect()
}


fn get_movie_folder_structure(movie_definition: &MovieDefinition) -> String {
  let movie_name = movie_definition.name();
  let tvdb_id = movie_definition.tvdb_id();
  format!("{movie_name} - {{tvdb-{tvdb_id}}} [tvdbid-{tvdb_id}]")
}


fn get_movie_directory(encodes_dir: &EncodesDir, movie_definition: &MovieDefinition) -> PathBuf {
  let movie_folder_structure = get_movie_folder_structure(movie_definition);
  encodes_dir.join(movie_folder_structure)
}
