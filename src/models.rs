use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use serde::Deserialize;


#[derive(Debug)]
pub struct Rename {
  pub from_file_name: PathBuf,
  pub to_file_name: PathBuf,
}

impl Rename {
  pub fn new(from: PathBuf, to: PathBuf) -> Self {
    Self {
      from_file_name: from,
      to_file_name: to,
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FileNameAndExt {
  pub path: PathBuf,
  pub file_name: String,
  pub ext: String
}

impl FileNameAndExt {
  pub fn new(path: &Path, file_name: &OsStr, ext: &OsStr) -> Self {
    Self {
      path: path.to_path_buf(),
      file_name: file_name.to_string_lossy().to_string(),
      ext: ext.to_string_lossy().to_string()
    }
  }
}

impl PartialOrd for FileNameAndExt {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path.partial_cmp(&other.path)
    }
}


#[derive(Debug)]
pub struct RipsDir(pub PathBuf);

impl AsRef<Path> for RipsDir {
    fn as_ref(&self) -> &Path {
      self.0.as_path()
    }
}

#[derive(Debug)]
pub struct EpisodeGuide(pub PathBuf);

#[derive(Debug)]
pub struct EncodesDir(pub PathBuf);

impl AsRef<Path> for EncodesDir {
    fn as_ref(&self) -> &Path {
      self.0.as_path()
    }
}


#[derive(Debug)]
pub struct RenamesDir(pub PathBuf);

#[derive(Debug)]
pub struct ProcessingDir(pub PathBuf);

impl ProcessingDir {
  pub fn rips_dir(&self) -> RipsDir {
    RipsDir(self.0.join("Rips"))
  }

  pub fn renames_dir(&self) -> RenamesDir {
    RenamesDir(self.0.join("Renames"))
  }

  pub fn encodes_dir(&self) -> EncodesDir {
    EncodesDir(self.0.join("Encodes"))
  }
}

impl AsRef<Path> for ProcessingDir {
  fn as_ref(&self) -> &Path {
    self.0.as_path()
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SeriesMetaData {
  pub name: String,
  pub tvdb_id: String,
  pub season_number: String,
}


#[derive(Debug, Deserialize, PartialEq)]
pub struct EpisodesDefinition {
  pub metadata: SeriesMetaData,
  pub episodes: Vec<EpisodeDefinition>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EpisodeDefinition {
  pub number: String,
  pub name: String,
}


pub enum RenamesResult {
  Correct,
  Wrong
}
