use crate::cli::MkvCommands;
use crate::html_scraper;
use crate::models::ROutput;
use super::rename_workflow;
use super::super::export_workflow;

pub async fn perform(movie_command: MkvCommands) -> ROutput {
  match movie_command {
    MkvCommands::Rename(rename_args) => rename_workflow::perform(rename_args).await,
    MkvCommands::Export(export_args) => export_workflow::perform(export_args, html_scraper::get_movie_definition).await,
  }
}
