mod models;
mod cli;
mod metadata_downloader;
mod html_scraper;
mod workflow;

use cli::get_cli_args;
use workflow::perform_workflow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let config = get_cli_args();
  perform_workflow(config).await
}
