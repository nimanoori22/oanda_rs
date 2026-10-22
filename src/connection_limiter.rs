use std::{
    future::{poll_fn, Future},
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll},
};
use tokio::{
    sync::Mutex,
    time::{sleep_until, Duration, Instant},
};
use tower_layer::Layer;
use tower_service::Service;

// All OandaClient instances in this process share the connection-start clock.
static NEXT_CONNECTION: OnceLock<Arc<Mutex<Instant>>> = OnceLock::new();

#[derive(Clone)]
pub(crate) struct ConnectionLimitLayer;

impl<S> Layer<S> for ConnectionLimitLayer {
    type Service = ConnectionLimit<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ConnectionLimit {
            inner,
            next: NEXT_CONNECTION
                .get_or_init(|| Arc::new(Mutex::new(Instant::now())))
                .clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ConnectionLimit<S> {
    inner: S,
    next: Arc<Mutex<Instant>>,
}

impl<S, R> Service<R> for ConnectionLimit<S>
where
    S: Service<R> + Clone + Send + 'static,
    S::Future: Send,
    S::Response: Send,
    S::Error: Send,
    R: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: R) -> Self::Future {
        let mut inner = self.inner.clone();
        let next = self.next.clone();
        Box::pin(async move {
            poll_fn(|cx| inner.poll_ready(cx)).await?;
            let mut next = next.lock().await;
            sleep_until(*next).await;
            *next = Instant::now() + Duration::from_millis(500);
            let connecting = inner.call(request);
            drop(next);
            connecting.await
        })
    }
}
