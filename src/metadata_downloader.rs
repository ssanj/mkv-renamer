use crate::models::RenamerError;

pub async fn download_metadata(url: &str) -> Result<String, RenamerError> {
  let body =
    reqwest::get(url)
      .await
      .map_err(|e| RenamerError::CouldNotAccessMetadataURL(url.to_owned(), e.to_string()))?
      .text()
      .await
      .map_err(|e| RenamerError::CouldNotDecodeMetadataBody(url.to_owned(), e.to_string()))?;

  Ok(body)
}
