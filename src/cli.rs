use clap::Parser;

/// Rename TV series ripped from optical media
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct Args {

  /// The location of series metadata file.
  /// An example format can be found at: https://raw.githubusercontent.com/ssanj/mkv-renamer/main/sample.conf
  #[clap(short, long, value_parser)]
  pub series_metadata: String,

  /// The location of the processing directory (PD).
  /// Structure: PD/{Rips, Renames, Encodes}
  #[clap(short, long, value_parser)]
  pub processing_dir: String,
}


pub fn get_cli_args() -> Args {
  Args::parse()
}
