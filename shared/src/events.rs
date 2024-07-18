use std::fmt::Debug;

use deadpool_redis::redis::{self, Commands};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use serde::{Deserialize, Serialize};

pub struct PubSub {
    channel: String,
    url: String,
}

impl PubSub {
    pub fn new(channel: &String, url: &String) -> Self {
        Self {
            channel: channel.clone(),
            url: url.clone(),
        }
    }

    pub fn open<M>(
        &self,
        mut rx: oneshot::Receiver<()>,
        tx_events: mpsc::Sender<M>,
    ) -> anyhow::Result<JoinHandle<()>>
    where
        M: Message + core::marker::Send + 'static,
    {
        let url = self.url.clone();
        let channel = self.channel.clone();

        let handle = tokio::spawn(async move {
            let mut conn = redis::Client::open(url)
                .expect("failed")
                .get_connection()
                .expect("failed");
            let mut pubsub = conn.as_pubsub();
            pubsub
                // Timemout does not matter as we make infinite loop
                .set_read_timeout(Some(std::time::Duration::from_secs(1)))
                .unwrap();
            // TODO: do we need to unsubscribe?
            pubsub.subscribe(channel).unwrap();

            loop {
                if rx.try_recv().is_ok() {
                    break;
                }

                let msg = pubsub.get_message();
                match msg {
                    Ok(msg) => {
                        let msg = Self::parse_msg(
                            msg.get_payload::<String>().unwrap(),
                        )
                        .unwrap();

                        tx_events.send(msg).await.unwrap();
                    }
                    Err(_e) => {
                        // TODO: handle error somehow
                        continue;
                    }
                }
            }
        });

        Ok(handle)
    }

    pub fn publish_message<M>(&self, message: M) -> anyhow::Result<()>
    where
        M: Message,
    {
        tracing::info!("Publishing message: {:?}", message);
        let mut conn = redis::Client::open(self.url.clone())
            .expect("failed")
            .get_connection()
            .expect("failed");

        let asd = serde_json::to_string(&message).unwrap();

        conn.publish(&self.channel, asd)?;

        Ok(())
    }

    fn parse_msg<M>(msg: String) -> anyhow::Result<M>
    where
        M: Message,
    {
        let obj: M = serde_json::from_str(&msg)?;

        Ok(obj)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PosInitializedMsg {
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestMsg {
    pub data: String,
}

pub trait Message: Serialize + for<'a> Deserialize<'a> + Debug {}

#[derive(Serialize, Deserialize, Debug)]
pub enum Messages {
    PosInitialized(PosInitializedMsg),
    Test(TestMsg),
}

impl Message for Messages {}
