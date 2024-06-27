use std::fmt;
use std::path::PathBuf;
use console::Style;

#[derive(Debug)]
pub enum RenamerError {
  InvalidMetadataConfiguration(String),
  CouldNotAccessMetadataFile(String, String),
  CouldNotAccessMetadataURL(String, String),
  CouldNotDecodeMetadataBody(String, String),
  CouldNotExportEpisodeMetadata(String, PathBuf, String),
  ProcessingDirectoryDoesNotExist(PathBuf),
  ProcessingDirAndMetadaPathDoesNotExit(PathBuf, PathBuf),
  MetadataDirectoryDoesNotExist(PathBuf),
  CouldNotDecodeEpisodeJson(PathBuf, String),
  NotEnoughMetadataForEpisodes(usize, usize),
  NoFilesToRename,
  CouldNotCreatedSeriesDirectory(PathBuf, String),
  SeriesDirectoryAlreadyExists(PathBuf),
}

impl std::error::Error for RenamerError {}

impl fmt::Display for RenamerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let red = Style::new().red();
      let error = match self {
        RenamerError::InvalidMetadataConfiguration(message) => format!("Invalid metadata configuration: {message}"),
        RenamerError::CouldNotAccessMetadataFile(file, message) => format!("Could not access metadata file: {file}, due to: {message}"),
        RenamerError::CouldNotAccessMetadataURL(url, message) => format!("Could not access metadata URL: {url}, due to: {message}"),
        RenamerError::CouldNotDecodeMetadataBody(url, message) => format!("Could not decode metadata body from URL: {url}, due to: {message}"),
        RenamerError::CouldNotExportEpisodeMetadata(url, path, message) => format!("Could not export metadata from URL: {url} to file: {}, due to: {message}", path.to_string_lossy()),
        RenamerError::ProcessingDirectoryDoesNotExist(path) => format!("Processing directory does not exist: {}", path.to_string_lossy()),
        RenamerError::ProcessingDirAndMetadaPathDoesNotExit(processing_dir, metadata_dir) => format!("Processing directory: {} and metadata path: {} does not exist", processing_dir.to_string_lossy(), metadata_dir.to_string_lossy()),
        RenamerError::MetadataDirectoryDoesNotExist(metadata_dir) => format!("Metadata path: {} does not exist", metadata_dir.to_string_lossy()),
        RenamerError::CouldNotDecodeEpisodeJson(path, message) => format!("Could not decode JSON from metadata file: {}, due to: {}", path.to_string_lossy(), message),
        RenamerError::NotEnoughMetadataForEpisodes(metadata, episodes) => format!("Not enough metadata episode names ({}) to match ripped files ({})", metadata, episodes),
        RenamerError::NoFilesToRename => "No files found to rename".to_owned(),
        RenamerError::CouldNotCreatedSeriesDirectory(path, message) => format!("Could not create series directory: {}, due to: {}", path.to_string_lossy(), message),
        RenamerError::SeriesDirectoryAlreadyExists(path) => format!("Series directory: {} already exists. Aborting.", path.to_string_lossy()),
      };

      write!(f, "{}", red.apply_to(error))
    }
}
