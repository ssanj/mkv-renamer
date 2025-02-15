use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use pretty_assertions::assert_eq;
use std::io::{Read, Write};

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn returns_version() -> Result<(), Box<dyn std::error::Error>> {
  let mut cmd = Command::cargo_bin("mkv-renamer").unwrap();

  let version = env!("CARGO_PKG_VERSION");
  let expected_version_string = format!("mkv-renamer {}\n", version);

  cmd
    .arg("-V")
    .assert()
    .success()
    .stdout(expected_version_string);

  Ok(())
}

// TODO: Add a similar test for series
#[test]
fn renames_movie_from_definition_file() -> Result<(), Box<dyn std::error::Error>> {
    //Inside processing dir:
    //Rips
    // - session1
    //  - disc1
    //   - DVD_TS_01.mkv
    //  - disc2
    //  - discx
    //  - renames
    //    - encodes_dir.txt
    //    - movie-name - {tvdb-number}.mkv
    // - session2
    // - sessionx
    //Encodes
    // - Series1
    //  - SO1E1 - blah - {tvdb-number}
    //  - SO1E2 - blee - {tvdb-number}
    //  - SO1E3 - blee - {tvdb-number}
    // - Movie1
    //  - Movie 1 - {tvdb-number}

    let processing_dir = tempdir().unwrap();
    let processing_dir_path = processing_dir.path();

    let rips = processing_dir_path.join("Rips");
    let encodes = processing_dir_path.join("Encodes");

    let mut created_dirs: Vec<PathBuf> =
      vec![
        rips.clone(),
        encodes.clone(),
      ];

    let movie_definition =
      r#"{
        "name":"The Big Lebowski",
        "tvdb_id":"659"
      }"#;


    let movie_definition_file_path = processing_dir_path.join("movie.conf");
    let mut movie_definition_file = File::create(&movie_definition_file_path)?;
    movie_definition_file.write_all(movie_definition.as_bytes())?;
    created_dirs.push(movie_definition_file_path.clone());

    create_all_directories(&encodes)?;

    for s in 1..=3 {
      let session = rips.join(format!("session{}", s));
      create_all_directories(&session)?;
      created_dirs.push(session.clone());
      for d in 1..=5 {
        let disc = session.join(format!("disc{}", d));
        create_all_directories(&disc)?;
        created_dirs.push(disc.clone());

        if s == 1 && d == 1 {
          let dvd_file = disc.join("DVD_TS_01.mkv");
          let _ = File::create(&dvd_file)?;
          created_dirs.push(dvd_file)
        }
      }

      let renames = session.join("renames");
      create_all_directories(&renames)?;
      created_dirs.push(renames);
    }

    for d in created_dirs {
      println!("called for: {}", d.to_string_lossy());
      assert!(d.exists(), "created dir: {} does not exist", d.to_string_lossy())
    }


    let mut cmd = Command::cargo_bin("mkv-renamer").unwrap();
    cmd
      .arg("movie")
      .arg("rename")
      .arg("-p")
      .arg(&processing_dir_path)
      .arg("-s")
      .arg("1")
      .arg("-f")
      .arg(&movie_definition_file_path)
      .write_stdin("y")
      .assert()
      .success();

    let encodes_movie_dir = encodes.join("The Big Lebowski - {tvdb-659}");
    let renames_dir = rips.join("session1").join("renames");
    let renamed_file = renames_dir.join("The Big Lebowski - {tvdb-659}.mkv");

    let encodes_file = renames_dir.join("encode_dir.txt");

    let expected_files: Vec<PathBuf> =
      vec![
        encodes_movie_dir.clone(),
        renamed_file,
        encodes_file.clone()
      ];


    for f in expected_files {
      assert!(&f.exists(), "{} does not exist", &f.to_string_lossy());
    }

    let mut encodes_file_handle = File::open(&encodes_file)?;
    let mut buffer = String::new();
    encodes_file_handle.read_to_string(&mut buffer)?;

    assert_eq!(buffer, encodes_movie_dir.to_string_lossy().to_string());

    Ok(())
}

fn create_all_directories(p: &Path) -> Result<(), Box<dyn std::error::Error>> {
  if !p.exists() {
    fs::create_dir_all(p)?;
  }

  Ok(())


}

