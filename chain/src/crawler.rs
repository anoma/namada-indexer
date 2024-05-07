use async_stream::stream;
use futures::pin_mut;
use futures_core::{stream::Stream, Future};
use futures_util::stream::StreamExt;
use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};
use tokio::signal;
use tokio_retry::{
    strategy::{jitter, FixedInterval},
    RetryIf,
};

use shared::block::BlockHeight;
use crate::error::MainError;

fn block_heights(
    from_height: u32,
    to_height: Option<u32>,
) -> impl Stream<Item = u32> {
    stream! {
        for i in from_height..to_height.unwrap_or(u32::MAX) {
            yield i;
        }
    }
}

pub async fn crawl<F, Fut>(
    f: F,
    last_block_height: BlockHeight,
) -> Result<(), MainError>
where
    F: Fn(u32) -> Fut,
    Fut: Future<Output = Result<(), MainError>>,
{
    let s = block_heights(last_block_height + 1, None);
    pin_mut!(s);
    let retry_strategy = FixedInterval::from_millis(5000).map(jitter);
    let must_exit = must_exit_handle();

    while let Some(block_height) = s.next().await {
        if must_exit.load(atomic::Ordering::Relaxed) {
            break;
        }
        _ = RetryIf::spawn(
            retry_strategy.clone(),
            || async {
                f(block_height).await?;
                Ok(())
            },
            |e: &MainError| {
                !must_exit.load(atomic::Ordering::Relaxed)
                    && (e.eq(&MainError::RpcError)
                        || e.eq(&MainError::Database))
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
