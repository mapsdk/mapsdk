use std::future::Future;

use lazy_static::lazy_static;
use tokio::{runtime::Runtime, task::JoinHandle};

lazy_static! {
    static ref TOKIO_RUNTIME: Runtime = Runtime::new().expect("Failed to create threaded runtime");
}

pub fn spawn<F: Future>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    TOKIO_RUNTIME.spawn(future)
}
