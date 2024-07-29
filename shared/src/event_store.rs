use deadpool_redis::{
    redis::{
        streams::{StreamReadOptions, StreamReadReply},
        AsyncCommands, Value,
    },
    Connection,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

const EVENT_STORE: &str = "event_store";
const EVENT: &str = "event";

pub async fn subscribe<T>(
    mut redis_conn: Connection,
    (last_processed_key, last_processed_val): (String, String),
    tx: mpsc::Sender<T>,
    mut exit_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()>
where
    T: SupportedEvents,
{
    let opts = StreamReadOptions::default().count(1).block(0);
    let mut last_processed_id = last_processed_val.clone();

    loop {
        let result: Option<StreamReadReply> = redis_conn
            .xread_options(&[EVENT_STORE], &[&last_processed_id], &opts)
            .await?;

        if let Some(reply) = result {
            for stream_key in reply.keys {
                for stream_id in stream_key.clone().ids {
                    tracing::info!("Processing event: {:?}", stream_key);
                    let event =
                        stream_id.map.get(EVENT).expect("event key not found");

                    match event {
                        Value::Data(data) => {
                            let event_str = std::str::from_utf8(data)
                                .expect("event is not valid utf8");

                            let event = T::from_stored(event_str);

                            redis_conn
                                .set(&last_processed_key, stream_id.id.clone())
                                .await?;
                            last_processed_id = stream_id.id;

                            tx.send(event).await.expect("send failed");
                        }
                        _ => {}
                    }
                }
            }
        }

        if exit_rx.try_recv().is_ok() {
            break;
        }
    }

    Ok(())
}

pub async fn publish<T>(
    mut redis_conn: Connection,
    event: T,
) -> anyhow::Result<()>
where
    T: SupportedEvents,
{
    let stored_event = event.to_stored();
    redis_conn.xadd(EVENT_STORE, "*", &[stored_event]).await?;

    Ok(())
}

// Define a macro to create event types with optional fields
macro_rules! define_event {
    // Case with fields
    ($name:ident, $($field_name:ident: $field_type:ty),*) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name {
            $(pub $field_name: $field_type),*
        }
    };


    // Case without fields
    ($name:ident) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name;

        impl Event for $name {}
    };
}

pub trait Event: Clone + Serialize + Send + Sync {
    fn name(&self) -> &'static str {
        // Get the type name at runtime and strip the module path
        let full_name = std::any::type_name::<Self>();
        // Find the last occurrence of "::" and return the substring after it
        match full_name.rsplit("::").next() {
            Some(name) => name,
            None => full_name,
        }
    }

    // Default implementation of payload
    fn payload(&self) -> Self {
        self.clone()
    }
}

define_event!(PosInitializedEventV1);
define_event!(ChainInitializedEventV1);
define_event!(Test);

pub trait SupportedEvents: for<'a> Deserialize<'a> + Serialize {
    fn from_stored(value: &str) -> Self {
        serde_json::from_str(value).unwrap()
    }

    fn to_stored(&self) -> (String, String) {
        let value = serde_json::to_string(&self).unwrap();

        ("event".to_string(), value)
    }
}

// TODO: move this to POS module
#[derive(Serialize, Deserialize, Debug)]
pub enum PosEvents {
    PosInitializedEventV1(PosInitializedEventV1),
    ChainInitializedEventV1(ChainInitializedEventV1),
    Test(Test),
}

impl SupportedEvents for PosEvents {}
