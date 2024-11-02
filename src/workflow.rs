use crate::models::*;
use crate::cli::*;

mod series;
mod movie;
mod common;
mod export_workflow;

pub use series::series_workflow as series_workflow;
pub use movie::movie_workflow as movie_workflow;

pub async fn perform_workflow(config: MkvRenamerArgs) -> ROutput {
  match config.commands {
    MkvInputType::Series(series_command) => series_workflow::perform(series_command).await,
    MkvInputType::Movie(movie_command) => movie_workflow::perform(movie_command).await,
  }
}
