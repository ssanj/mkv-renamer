use clap::{Args, Parser};

/// Rename TV series ripped from makeMKV
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct MkvRenamerArgs {

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

  /// The session to use, accepts values from 1 to 100.
  #[clap(short, long, value_parser=clap::value_parser!(u8).range(1..100))]
  pub session_dir: u8,

  /// Verbose logging
  #[clap(long, value_parser)]
  pub verbose: bool,
}

#[derive(Args, Clone, Debug)]
#[group(required = true, multiple = false)]
pub struct MetadataInputType {

    /// The url of TVDB season information.
    /// Example: https://thetvdb.com/series/thundercats/seasons/official/1
    #[arg(long, short, value_name = "url")]
    pub url_metadata: Option<String>,

    /// The location of series metadata file.
    /// An example format can be found at: https://raw.githubusercontent.com/ssanj/mkv-renamer/main/sample.conf
    #[arg(long, short, value_name = "file")]
    pub file_metadata: Option<String>,
}


pub fn get_cli_args() -> MkvRenamerArgs {
  MkvRenamerArgs::parse()
}
