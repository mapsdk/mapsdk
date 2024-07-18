use std::error::Error;

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Response,
};

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get(&self, url: &str) -> Result<Response, Box<dyn Error>> {
        let resp = self.client.get(url).send().await?;
        if let Err(err) = resp.error_for_status_ref() {
            return Err(err.into());
        }

        Ok(resp.into())
    }

    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: &Vec<(&'static str, &'static str)>,
    ) -> Result<Response, Box<dyn Error>> {
        let mut header_map = HeaderMap::new();
        for (k, v) in headers.iter() {
            header_map.insert(*k, HeaderValue::from_static(v));
        }

        let resp = self.client.get(url).headers(header_map).send().await?;
        if let Err(err) = resp.error_for_status_ref() {
            return Err(err.into());
        }

        Ok(resp.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get() {
        let http_client = HttpClient::new();
        let image = http_client
            .get("https://www.baidu.com/img/bd_logo1.png")
            .await;
        assert!(image.is_ok());
    }

    #[tokio::test]
    async fn test_get_with_headers() {
        let headers = vec![("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"),("Accept", "image/webp,image/apng,image/*,*/*;q=0.8"),("Accept-Encoding", "gzip, deflate, br")];

        let http_client = HttpClient::new();
        let image = http_client
            .get_with_headers("http://a.tile.osm.org/0/0/0.png", &headers)
            .await;
        assert!(image.is_ok());
    }
}
