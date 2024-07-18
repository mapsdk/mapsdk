use std::{future::Future, sync::Arc};

use tokio::{runtime::Runtime, task::JoinHandle};

use crate::http::HttpClient;

pub struct Env {
    tokio_runtime: Runtime,
    http_client: Arc<HttpClient>,
}

impl Env {
    pub fn new() -> Self {
        let _ = env_logger::try_init();

        Self {
            tokio_runtime: Runtime::new().expect("Failed to create threaded runtime"),
            http_client: Arc::new(HttpClient::new()),
        }
    }

    pub fn get_http_client(&self) -> Arc<HttpClient> {
        self.http_client.clone()
    }

    pub fn spawn<F: Future>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tokio_runtime.spawn(future)
    }
}
