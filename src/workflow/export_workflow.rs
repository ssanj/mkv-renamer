use std::path::PathBuf;
use serde::Serialize;

use crate::metadata_downloader::download_metadata;
use crate::models::*;
use crate::cli::ExportArgs;

pub async fn perform<I: Serialize, F: FnOnce(&str) -> I>(export_args: ExportArgs, get_metadata: F) -> ROutput {
  let url = export_args.url_metadata;
  let export_path = export_args.export_path;
  handle_url_metadata_export(&url, get_metadata, export_path).await
}

async fn handle_url_metadata_export<I: Serialize, F: FnOnce(&str) -> I>(url: &str, get_metadata: F, export_path: PathBuf) -> ROutput {
  let page_content = download_metadata(url).await?;

  // TODO: Change for movie
  let episodes_definition = get_metadata(&page_content);
  use std::fs::OpenOptions;

  OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(&export_path)
    .map_err(|e| {
      // TODO: Change for movie
      RenamerError::CouldNotExportEpisodeMetadata(url.to_owned(), export_path.clone(), e.to_string())
    })
    .and_then(|file| {
      // TODO: Change for movie - parameterise
      serde_json::to_writer_pretty(file, &episodes_definition)
        .map_err(|e| {
          // TODO: Change for movie
          RenamerError::CouldNotExportEpisodeMetadata(url.to_owned(), export_path.clone(), e.to_string())
        })
    })
    .map(|_| Output::Success)
}
