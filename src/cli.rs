use clap::{Args, Parser};

/// Rename TV series ripped from optical media
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub struct MkvRenamerArgs {

  #[command(flatten)]
  pub metadata_input_type: MetadataInputType,

  /// The location of the processing directory (PD).
  /// Structure: PD/{Rips, Renames, Encodes}
  #[clap(short, long, value_parser)]
  pub processing_dir: String,
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
