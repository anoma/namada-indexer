use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};

use async_stream::stream;
use futures::pin_mut;
use futures_core::Future;
use futures_core::stream::Stream;
use futures_util::stream::StreamExt;
use tokio::signal;
use tokio_retry::RetryIf;
use tokio_retry::strategy::{FixedInterval, jitter};

use crate::error::MainError;

fn indexes(from: u32, to: Option<u32>) -> impl Stream<Item = u32> {
    stream! {
        for i in from..to.unwrap_or(u32::MAX) {
            yield i;
        }
    }
}

pub async fn crawl<F, Fut>(
    f: F,
    first_index: u32,
    interval: Option<u64>,
) -> Result<(), MainError>
where
    F: Fn(u32) -> Fut,
    Fut: Future<Output = Result<(), MainError>>,
{
    let interval = interval.unwrap_or(5000);
    let s = indexes(first_index, None);
    pin_mut!(s);
    let retry_strategy = FixedInterval::from_millis(interval).map(jitter);
    let must_exit = must_exit_handle();

    while let Some(index) = s.next().await {
        if must_exit.load(atomic::Ordering::Relaxed) {
            break;
        }
        _ = RetryIf::spawn(
            retry_strategy.clone(),
            || async {
                f(index).await?;
                Ok(())
            },
            |e: &MainError| {
                !must_exit.load(atomic::Ordering::Relaxed)
                    && (e.eq(&MainError::RpcError)
                        || e.eq(&MainError::Database)
                        || e.eq(&MainError::NoAction))
            },
        )
        .await;
    }

    Ok(())
}

fn must_exit_handle() -> Arc<AtomicBool> {
    let handle = Arc::new(AtomicBool::new(false));
    let task_handle = Arc::clone(&handle);
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Error receiving interrupt signal");
        task_handle.store(true, atomic::Ordering::Relaxed);
    });
    handle
}
