use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Rename TV series ripped from makeMKV
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct MkvRenamerArgs {
  #[command(subcommand)]
  pub commands: MkvInputType
}

#[derive(Debug, Clone, Subcommand)]
pub enum MkvInputType {

  /// Process TV Series
  #[command(subcommand)]
  Series(MkvCommands),

  /// Process Movies
  #[command(subcommand)]
  Movie(MkvCommands)
}


#[derive(Debug, Clone, Subcommand)]
pub enum MkvCommands {
  /// Renames a collection of ripped episodes from a metadata source
  Rename(RenameArgs),

  /// Exports metadata information for a series to a file
  Export(ExportArgs),
}

#[derive(Args, Clone, Debug)]
pub struct RenameArgs {
  #[command(flatten)]
  pub metadata_input_type: MetadataInputType,

  /// The location of the processing directory (PD). See extended help for a full structure.
  ///
  /// Structure: PD/{Rips,Encodes}
  ///
  /// Structure: Rips/{session1,session2,sessionN}
  ///
  /// Structure: sessionX/{disc1,disc2,disc3,discN,renames}
  #[clap(short, long, value_parser)]
  pub processing_dir: String,

  /// The session number to use, accepts values from 1 to 100. The number maps to a session<SESSION_NUMBER> directory.
  #[clap(short, long, value_parser=clap::value_parser!(u8).range(1..100))]
  pub session_number: u8,

  /// Verbose logging
  #[clap(long, value_parser)]
  pub verbose: bool,

  /// Skips file checks.
  ///
  /// If this flag is specified, the encodes file will be created in the rename directory and the output directory will be created for the series or movie. No files will be renamed.
  #[clap(long, value_parser)]
  pub skip_files: bool,
}

#[derive(Args, Clone, Debug)]
pub struct ExportArgs {
  /// The url of TVDB season information.
  /// Example: https://thetvdb.com/series/thundercats/seasons/official/1
  #[arg(long, short, value_name = "url")]
  pub url_metadata: String,

  /// Where to extract the metadata to
  #[arg(long, short, value_name = "path")]
  pub export_path: PathBuf
}

#[derive(Clone, Debug, ValueEnum)]
pub enum InputType {
  Series,
  Movie
}


#[derive(Args, Clone, Debug)]
#[group(required = true, multiple = false)]
pub struct MetadataInputType {

  /// The url of TVDB season information.
  /// Example: https://thetvdb.com/series/thundercats/seasons/official/1
  #[arg(long, short, value_name = "url")]
  pub url_metadata: Option<String>,

  /// The location of series metadata file. This depends on the input_type specified
  /// An example formats can be found at: https://raw.githubusercontent.com/ssanj/mkv-renamer/main/series-sample.conf
  /// and https://raw.githubusercontent.com/ssanj/mkv-renamer/main/movie-sample.conf
  #[arg(long, short, value_name = "file")]
  pub file_metadata: Option<String>
}


pub fn get_cli_args() -> MkvRenamerArgs {
  MkvRenamerArgs::parse()
}
