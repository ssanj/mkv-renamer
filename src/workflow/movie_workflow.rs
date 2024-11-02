use crate::cli::MkvCommands;
use crate::models::ROutput;
use super::{export_workflow, rename_workflow};

pub async fn perform(movie_command: MkvCommands) -> ROutput {
  match movie_command {
    MkvCommands::Rename(rename_args) => todo!(),
    MkvCommands::Export(export_args) => todo!(),
  }
}
