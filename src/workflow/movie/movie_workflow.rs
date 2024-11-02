use crate::cli::MkvCommands;
use crate::models::ROutput;
use super::rename_workflow;

pub async fn perform(movie_command: MkvCommands) -> ROutput {
  match movie_command {
    MkvCommands::Rename(rename_args) => rename_workflow::perform(rename_args).await,
    MkvCommands::Export(export_args) => todo!(),
  }
}
