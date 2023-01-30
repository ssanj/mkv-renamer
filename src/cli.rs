use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct Args {

  /// The location of series configuration file
  #[clap(short, long, value_parser)]
  pub config_file: String,

  /// The location of .mkv to be renamed
  #[clap(short, long, value_parser)]
  pub dvd_rips: String,

  /// The location where the renamed files are written to
  #[clap(short, long, value_parser)]
  pub renames_directory: String
}


pub fn get_cli_args() -> Args {
  Args::parse()
}
