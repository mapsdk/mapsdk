use std::error::Error;

use image::{load_from_memory, DynamicImage};

use crate::http::HttpClient;

pub async fn image_from_url(
    http_client: &HttpClient,
    url: &str,
    headers: &Vec<(&'static str, &'static str)>,
) -> Result<DynamicImage, Box<dyn Error>> {
    let resp = http_client.get_with_headers(url, headers).await?;
    let bytes = resp.bytes().await?;

    Ok(load_from_memory(&bytes)?)
}
