use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct RipsDir(pub PathBuf);

impl RipsDir {
  pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
    self.0.join(path)
  }
}

impl AsRef<Path> for RipsDir {
    fn as_ref(&self) -> &Path {
      self.0.as_path()
    }
}

#[derive(Debug)]
pub struct RipsSessionDir(pub PathBuf);

impl AsRef<Path> for RipsSessionDir {
    fn as_ref(&self) -> &Path {
      self.0.as_path()
    }
}

#[allow(dead_code)]
impl RipsSessionDir {
  pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
    self.0.join(path)
  }
}

#[derive(Debug)]
pub struct RipsSessionRenamesDir(pub PathBuf);

impl AsRef<Path> for RipsSessionRenamesDir {
  fn as_ref(&self) -> &Path {
    self.0.as_path()
  }
}


#[derive(Debug)]
pub struct EncodesDir(pub PathBuf);

impl EncodesDir {
  pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
    self.0.join(path)
  }
}


impl AsRef<Path> for EncodesDir {
    fn as_ref(&self) -> &Path {
      self.0.as_path()
    }
}


#[derive(Debug)]
pub struct SessionDir(pub String);

impl SessionDir {
  pub fn new(session_id: u8) -> Self {
    Self(format!("session{session_id}"))
  }

  /// sessionX/renames
  pub fn renames_dir(&self) -> PathBuf {
    Path::new(&self.0).join("renames").to_owned()
  }
}

impl AsRef<Path> for SessionDir {
  fn as_ref(&self) -> &Path {
    Path::new(&self.0)
  }
}

#[derive(Debug)]
pub struct ProcessingDir(pub PathBuf);

impl ProcessingDir {
  /// PD/Rips
  fn rips_dir(&self) -> RipsDir {
    RipsDir(self.0.join("Rips"))
  }

  /// PD/Rips/sessionX
  pub fn rips_session_dir(&self, session_dir: &SessionDir) -> RipsSessionDir {
    RipsSessionDir(self.rips_dir().join(session_dir))
  }

  /// PD/Rips/sessionX/renames
  pub fn rips_session_renames_dir(&self, session_dir: &SessionDir) -> RipsSessionRenamesDir {
    RipsSessionRenamesDir(self.rips_dir().join(session_dir.renames_dir()))
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
