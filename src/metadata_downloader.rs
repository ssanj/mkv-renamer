pub async fn download_metadata(url: &str) -> Result<String, Box<dyn std::error::Error>> {
  let body =
    reqwest::get(url)
      .await?
      .text()
      .await?;


  Ok(body)
}
