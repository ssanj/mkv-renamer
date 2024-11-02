use crate::cli::MkvCommands;
use crate::models::ROutput;
use super::{export_workflow, rename_workflow};

pub async fn perform(series_command: MkvCommands) -> ROutput {
  match series_command {
    MkvCommands::Rename(rename_args) => rename_workflow::perform(rename_args).await,
    MkvCommands::Export(export_args) => export_workflow::perform(export_args).await,
  }
}
