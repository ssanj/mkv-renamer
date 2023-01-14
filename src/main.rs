use walkdir::WalkDir;
use std::fmt;
use std::path::Path;
use std::fs;
use std::io::BufRead;

fn main() {

  let working_dir = "/Volumes/MediaDrive/TV_Rips/"; //current dir
  let target_dir = "/Volumes/MediaDrive/TV/";

  #[derive(Debug)]
  struct Episode {
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
    fn new(num: &str, desc: &str, tv: &str) -> Self {
      Self {
        number: num.to_owned(),
        description: desc.to_owned(),
        tvdb: tv.to_owned()
      }
    }
  }

  #[derive(Debug)]
  struct Rename {
    from_file_name: String,
    to_file_name: String,
  }

  impl Rename {
    fn new(from: &str, to: &str) -> Self {
      Self {
        from_file_name: from.to_owned(),
        to_file_name: to.to_owned(),
      }
    }
  }

  let episode_names =
    vec![
      Episode::new("S04E01", "Tattered and Torn", "81670"),
      Episode::new("S04E02", "Kommando", "81670"),
      Episode::new("S04E03", "Buffalo Shuffle", "81670"),
      Episode::new("S04E04", "Downstairs Upstairs", "81670"),
      Episode::new("S04E05", "Monsieur Murdoch", "81670"),
      Episode::new("S04E06", "Dead End Street", "81670"),
      Episode::new("S04E07", "Confederate Treasure", "81670"),
      Episode::new("S04E08", "Dial M for Murdoch", "81670"),
      Episode::new("S04E09", "The Black Hand", "81670"),
      Episode::new("S04E10", "Voices", "81670"),
      Episode::new("S04E11", "Bloodlust", "81670"),
      Episode::new("S04E12", "Kissing Bandit", "81670"),
      Episode::new("S04E13", "Murdoch in Wonderland", "81670"),
    ];

  let mut dirs: Vec<_> = WalkDir::new(working_dir)
      .into_iter()
      .filter_map(|re| re.ok())
      .filter(|dir_entry| dir_entry.path().is_file() && dir_entry.path().to_string_lossy().to_string().contains("/disk"))
      .map(|dir_entry| {
        dir_entry.into_path().into_os_string().into_string().unwrap()
      })
      // .filter_map(|dir_entry|{
      //   println!("dir_entry: {:?}", dir_entry.path());
      //   if dir_entry.path().is_dir() && dir_entry.path().starts_with("./target") {
      //     Some(dir_entry.path().to_string_lossy().to_string())
      //   } else {
      //     None
      //   }
      // })
      .collect();

  dirs.sort();

  let files_to_rename: Vec<_> =
    dirs
      .iter()
      .enumerate()
      .map(|(i, original_file_name)|{
        let p = Path::new(original_file_name);
        let ext = p.extension().map(|os| os.to_string_lossy()).expect(&format!("could not get extension for {}", p.to_string_lossy()));
        let episode = episode_names.get(i).expect(&format!("could not read episode_names index: {}", i));
        let file_name_with_ext = format!("{}.{}",episode, ext);
        let output_file_path = Path::new(target_dir).join(file_name_with_ext);
        let output_file_name = output_file_path.to_string_lossy().to_string();
        Rename::new(original_file_name, &output_file_name)
      })
      .collect();


  println!("The following renames will be performed:");
  for f in files_to_rename {
    println!("{} -> {}", f.from_file_name, f.to_file_name)
  }

  println!("");

  println!("Proceed with rename? 'y' to proceed or any other key to abort");
  let mut user_response = String::new();
  let stdin = std::io::stdin();
  let mut handle = stdin.lock();
  handle.read_line(&mut user_response).expect("Could not read from stdin"); // Unexpected, so throw
  let line = user_response.lines().next().expect("Could not extract line from buffer"); // Unexpected, so throw

  match line {
    "y" => println!("performing rename"),
    _ => println!("aborting rename")
  }

  // check the folder size and episode sizes are the same or at least there are more episodes than folders?
  // for (i, original_file_name) in dirs.iter().enumerate() {


  //   // fs::rename(original_file_name, &output_file_name).expect(&format!("could not rename {} -> {}", original_file_name, output_file_name))
  // }

}
