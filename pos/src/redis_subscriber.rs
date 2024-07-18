use std::sync::Arc;

use clap::Parser;
use deadpool_redis::redis::{self, Commands};
use tokio::{
    sync::{watch, Mutex},
    task::{self, JoinHandle},
};

use crate::config::AppConfig;

pub fn subscribe(channel: String) -> anyhow::Result<JoinHandle<()>> {
    let (_tx, rx) = watch::channel(false);

    let handle = tokio::spawn(async move {
        let config = AppConfig::parse();
        let conn = redis::Client::open(config.queue_url)
            .expect("failed")
            .get_connection()
            .expect("failed");
        let conn_arc = Arc::new(Mutex::new(conn));

        tracing::info!("Subscribed to channel: {}", channel);

        let _rx = rx.clone();
        loop {
            let conn = conn_arc.clone();
            let channel = channel.clone();

            tracing::info!("Waiting for message");
            let result = task::spawn_blocking(|| async move {
                tracing::info!("Blocking call started");
                let mut conn = conn.lock().await;

                let mut pubsub = conn.as_pubsub();
                pubsub.subscribe(channel.clone()).unwrap();

                pubsub
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();

                let msg = pubsub.get_message().unwrap();
                tracing::info!("Received message: {:?}", msg);

                pubsub.unsubscribe(channel).unwrap();

                msg
            });

            match result.await {
                Ok(res) => {
                    tracing::info!(
                        "Blocking call result: {:?}",
                        res.await.get_payload::<String>()
                    )
                }
                Err(e) => println!("Blocking call failed: {}", e),
            }

            // Check if we should stop
            if *rx.borrow() {
                break;
            }
        }
    });

    Ok(handle)
}

pub fn publish_message(channel: String, message: String) -> anyhow::Result<()> {
    let config = AppConfig::parse();
    let mut conn = redis::Client::open(config.queue_url)
        .expect("failed")
        .get_connection()
        .expect("failed");

    conn.publish(channel, message.clone())?;
    tracing::info!("Published message: {}", message);

    Ok(())
}
