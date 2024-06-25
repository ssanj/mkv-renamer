mod models;
mod cli;
mod metadata_downloader;
mod html_scraper;
mod workflow;

use cli::get_cli_args;
use console::style;
use workflow::perform_workflow;

#[tokio::main]
async fn main() {
  let config = get_cli_args();
  match perform_workflow(config).await {
    Ok(_) => println!("{}", style("Renaming completed successfully").green()),
    Err(e) => eprintln!("{}", style(e)),
  }
}
