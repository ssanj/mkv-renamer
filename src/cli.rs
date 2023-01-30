use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct Args {

  /// The location of series configuration file
  #[clap(short, long, value_parser)]
  config_file: String
}


pub fn get_cli_args() -> Args {
  Args::parse()
}
