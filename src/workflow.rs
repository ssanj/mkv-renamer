use crate::models::*;
use crate::cli::*;

mod rename_workflow;
mod export_workflow;
mod series_workflow;
mod movie_workflow;

pub async fn perform_workflow(config: MkvRenamerArgs) -> ROutput {
  match config.commands {
    MkvInputType::Series(series_command) => series_workflow::perform(series_command).await,
    MkvInputType::Movie(movie_command) => movie_workflow::perform(movie_command).await,
  }
}
