use std::path::PathBuf;
use std::error::Error;
use crate::html_scraper::get_series_metadata;
use crate::metadata_downloader::download_metadata;
use crate::models::*;
use crate::cli::ExportArgs;

pub async fn perform(export_args: ExportArgs) -> ROutput {
  let url = export_args.url_metadata;
  let export_path = export_args.export_path;
  handle_url_metadata_export(&url, export_path).await
}

async fn handle_url_metadata_export(url: &str, export_path: PathBuf) -> ROutput {
  let page_content = download_metadata(url).await?;
  let episodes_definition = get_series_metadata(&page_content);
  use std::fs::OpenOptions;

  OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(&export_path)
    .map_err(|e| {
      let err: Box<dyn Error> =
        Box::new(RenamerError::CouldNotExportEpisodeMetadata(url.to_owned(), export_path.clone(), e.to_string()));
      err
    })
    .and_then(|file| {
      serde_json::to_writer_pretty(file, &episodes_definition)
        .map_err(|e| {
          let err: Box<dyn Error> =
            Box::new(RenamerError::CouldNotExportEpisodeMetadata(url.to_owned(), export_path.clone(), e.to_string()));
          err
        })
    })
    .map(|_| Output::Success)
}
