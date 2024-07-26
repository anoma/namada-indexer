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
    last_processed_id: String,
    tx: mpsc::Sender<T>,
    mut exit_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let opts = StreamReadOptions::default().count(1).block(0);
    let mut last_processed_id = last_processed_id;

    loop {
        let result: Option<StreamReadReply> = redis_conn
            .xread_options(&[EVENT_STORE], &[&last_processed_id], &opts)
            .await?;

        if let Some(reply) = result {
            for stream_key in reply.keys {
                for stream_id in stream_key.ids {
                    let event =
                        stream_id.map.get(EVENT).expect("event key not found");

                    match event {
                        Value::Data(data) => {
                            let event_str = std::str::from_utf8(data)
                                .expect("event is not valid utf8");

                            let event = T::from_stored(event_str);

                            redis_conn
                                .set("last_processed_id", stream_id.id.clone())
                                .await?;
                            last_processed_id = stream_id.id;

                            tx.send(event).await?;
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
    T: Event,
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

        impl Event for $name {
            fn from_stored(value: &str) -> Self {
                serde_json::from_str(value).unwrap()
            }
        }
    };


    // Case without fields
    ($name:ident) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name;

        impl Event for $name {
            fn to_stored(&self) -> (String, String) {
                let key = self.name().to_string();
                let value = String::new(); // Return an empty string for structs without fields
                (key, value)
            }

            fn from_stored(value: &str) -> Self {
                serde_json::from_str(value).unwrap()
            }
        }
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

    fn to_stored(&self) -> (String, String) {
        let key = self.name().to_string();
        let value = serde_json::to_string(&self).unwrap();

        (key, value)
    }

    // Factory method to create an event from stored data
    fn from_stored(value: &str) -> Self
    where
        Self: Sized;
}

define_event!(PosInitializedEventV1);
define_event!(ChainInitializedEventV1);

trait Support

pub enum SupportedEvents {
    PosInitializedEventV1(PosInitializedEventV1),
    ChainInitializedEventV1(ChainInitializedEventV1),
}
