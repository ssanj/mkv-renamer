use std::fmt;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Series {
  name: String,
  season: String,
  tvdbId: String,
  episodes: Vec<Episode>
}


#[derive(Debug, Deserialize)]
pub struct Episode {
  number: String,
  description: String,
  tvdb: String,
}

impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {} {{tvdb-{}}}", self.number, self.description, self.tvdb)
    }
}

impl Episode {
  pub fn new(num: &str, desc: &str, tv: &str) -> Self {
    Self {
      number: num.to_owned(),
      description: desc.to_owned(),
      tvdb: tv.to_owned()
    }
  }
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

#[derive(Debug, PartialEq)]
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
