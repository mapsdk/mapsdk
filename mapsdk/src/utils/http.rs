use std::{cmp::Reverse, error::Error, fmt::Debug, hash::Hash, sync::Arc, time::SystemTime};

use bytes::Bytes;
use priority_queue::PriorityQueue;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Response,
};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
    time::{sleep, Duration},
};

use crate::env;

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
        headers: &Vec<(impl ToString, impl ToString)>,
    ) -> Result<Response, Box<dyn Error>> {
        let mut header_map = HeaderMap::new();
        for (k, v) in headers.iter() {
            header_map.insert(
                HeaderName::from_bytes((*k).to_string().as_bytes())?,
                HeaderValue::from_str(&v.to_string())?,
            );
        }

        let resp = self.client.get(url).headers(header_map).send().await?;
        if let Err(err) = resp.error_for_status_ref() {
            return Err(err.into());
        }

        Ok(resp.into())
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum HttpRequest<T: Clone + Debug + Eq + Hash + PartialEq> {
    Get {
        id: T,
        url: String,
        headers: Vec<(String, String)>,
    },
}

impl<T: Clone + Debug + Eq + Hash + PartialEq> HttpRequest<T> {
    pub fn id(&self) -> T {
        match self {
            HttpRequest::Get { id, .. } => id.clone(),
        }
    }
}

pub struct HttpResponse<T> {
    pub id: T,

    response: Response,
}

impl<T> HttpResponse<T> {
    pub async fn bytes(self) -> Result<Bytes, reqwest::Error> {
        self.response.bytes().await
    }
}

struct HttpRequestWorker {
    id: String,
    handle: JoinHandle<()>,
}

impl HttpRequestWorker {
    fn new<T: Clone + Debug + Eq + Hash + Send + 'static>(
        id: String,
        request_receiver: Arc<Mutex<mpsc::UnboundedReceiver<HttpRequest<T>>>>,
        response_sender: mpsc::UnboundedSender<HttpResponse<T>>,
    ) -> Self {
        let handle = env::spawn({
            let worker_id = id.clone();

            async move {
                let client = Client::new();

                loop {
                    let http_request = { request_receiver.lock().await.recv().await };

                    if let Some(http_request) = http_request {
                        log::info!(
                            "HttpRequestWorker[{}] received request: {:?}",
                            &worker_id,
                            http_request
                        );

                        let (id, resp) = match http_request {
                            HttpRequest::Get {
                                id, url, headers, ..
                            } => {
                                let mut header_map = HeaderMap::new();
                                for (k, v) in headers.iter() {
                                    if let (Ok(name), Ok(val)) = (
                                        HeaderName::from_bytes((*k).as_bytes()),
                                        HeaderValue::from_str(v),
                                    ) {
                                        header_map.insert(name, val);
                                    }
                                }

                                (id, client.get(url).headers(header_map).send().await)
                            }
                        };

                        match resp {
                            Ok(response) => {
                                let http_response = HttpResponse { id, response };
                                let _ = response_sender.send(http_response);
                            }
                            Err(err) => {
                                log::error!("{}", err);
                            }
                        }
                    }
                }
            }
        });

        Self { id, handle }
    }

    fn abort(&self) {
        self.handle.abort();
    }
}

pub struct HttpPool<T: Clone + Debug + Eq + Hash> {
    id: String,
    size: usize,

    request_queue: Arc<std::sync::Mutex<PriorityQueue<HttpRequest<T>, Reverse<SystemTime>>>>,
    request_workers: Vec<HttpRequestWorker>,
    response_sender: mpsc::UnboundedSender<HttpResponse<T>>,

    request_queue_handle: JoinHandle<()>,
}

impl<T: Clone + Debug + Eq + Hash> Drop for HttpPool<T> {
    fn drop(&mut self) {
        self.request_queue_handle.abort();

        self.request_workers.iter().for_each(|worker| {
            worker.abort();
        });
    }
}

impl<T: Clone + Debug + Eq + Hash + Send + 'static> HttpPool<T> {
    pub fn new(size: usize, response_sender: mpsc::UnboundedSender<HttpResponse<T>>) -> Self {
        assert!(size > 0);

        let id = nanoid::nanoid!();

        let (request_sender, request_receiver) = mpsc::unbounded_channel();
        let shared_request_receiver = Arc::new(Mutex::new(request_receiver));

        let mut request_workers = Vec::with_capacity(size);
        for i in 0..size {
            request_workers.push(HttpRequestWorker::new(
                format!("{}.{}", id, i),
                shared_request_receiver.clone(),
                response_sender.clone(),
            ));
        }

        let request_queue = Arc::new(std::sync::Mutex::new(PriorityQueue::new()));

        let request_queue_handle = env::spawn({
            let request_queue = request_queue.clone();

            async move {
                loop {
                    let mut request: Option<HttpRequest<T>> = None;

                    {
                        if let Ok(mut request_queue) = request_queue.lock() {
                            if let Some((req, _)) = request_queue.pop() {
                                request = Some(req);
                            }
                        }
                    }

                    match request {
                        Some(request) => {
                            let _ = request_sender.send(request);
                        }
                        None => sleep(Duration::from_millis(200)).await,
                    }
                }
            }
        });

        Self {
            id,
            size,

            request_queue,
            request_workers,
            response_sender,

            request_queue_handle,
        }
    }

    pub fn cancel(&self, id: &T) {
        if let Ok(mut request_queue) = self.request_queue.lock() {
            let requests: Vec<_> = request_queue
                .iter()
                .filter(|(request, _)| request.id() == *id)
                .map(|(request, _)| request.clone())
                .collect();

            requests.iter().for_each(|request| {
                request_queue.remove(request);
            })
        }
    }

    pub fn send(&self, request: HttpRequest<T>) {
        if let Ok(mut request_queue) = self.request_queue.lock() {
            request_queue.push(request, Reverse(SystemTime::now()));
        }
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
