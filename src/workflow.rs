use crate::models::*;
use crate::cli::*;

mod rename_workflow;
mod export_workflow;

pub async fn perform_workflow(config: MkvRenamerArgs) -> ROutput {
  match config.commands {
    MkvCommands::Rename(rename_args) => rename_workflow::perform(rename_args).await,
    MkvCommands::Export(export_args) => export_workflow::perform(export_args).await,
  }
}
