use std::{error::Error, future::Future};

use async_trait::async_trait;

#[async_trait]
pub trait Runtime {
    type Error: Error;

    type JoinHandle<T>: Future<Output = Result<T, Self::Error>>;

    fn spawn<F>(&self, task: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + 'static;

    async fn yield_now();
}
