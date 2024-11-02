use crate::{cli::MkvCommands, html_scraper};
use crate::models::ROutput;
use super::rename_workflow;
use super::super::export_workflow;

pub async fn perform(series_command: MkvCommands) -> ROutput {
  match series_command {
    MkvCommands::Rename(rename_args) => rename_workflow::perform(rename_args).await,
    MkvCommands::Export(export_args) => export_workflow::perform(export_args, html_scraper::get_series_metadata).await,
  }
}
