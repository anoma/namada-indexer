use std::{fmt::Debug, sync::Arc};

use deadpool_redis::redis::{self, Commands};
use tokio::sync::{mpsc, oneshot, Mutex};

use serde::{Deserialize, Serialize};

fn parse_msg<M>(msg: String) -> anyhow::Result<M>
where
    M: Message,
{
    let obj: M = serde_json::from_str(&msg)?;

    Ok(obj)
}

pub async fn subscribe<'a>(
    mut rx: oneshot::Receiver<()>,
    tx_events: mpsc::Sender<Messages>,
    mut pubsub: redis::PubSub<'a>,
    channel: &str,
) -> anyhow::Result<()> {
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
                let msg =
                    // TODO: unwraps
                    parse_msg(msg.get_payload::<String>().unwrap()).unwrap();

                tx_events.send(msg).await.unwrap();
            }
            Err(_e) => {
                // TODO: handle error somehow
                continue;
            }
        }
    }

    Ok(())
}

pub fn publish<M>(
    conn: &mut redis::Connection,
    channel: &str,
    message: M,
) -> anyhow::Result<()>
where
    M: Message,
{
    let message = serde_json::to_string(&message).unwrap();

    conn.publish(&channel, message)?;

    Ok(())
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
    ChainReady(TestMsg),
}

impl Message for Messages {}
