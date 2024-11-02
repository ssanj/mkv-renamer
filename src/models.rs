use std::path::{Path, PathBuf};
use std::ffi::OsStr;

mod dirs;
mod series;
mod movie;
mod errors;
pub use dirs::*;
pub use series::*;
pub use movie::*;
pub use errors::*;

pub type R = Result<(), RenamerError>;
pub type ROutput = Result<Output, RenamerError>;

pub enum Output {
  Success,
  UserCanceled
}

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

pub enum RenamesResult {
  Correct,
  Wrong
}
