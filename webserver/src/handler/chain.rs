use std::{convert::Infallible, time::Duration};

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
};
use futures::Stream;
use tokio_stream::StreamExt;

use crate::state::common::CommonState;

pub async fn sync_height(
    State(state): State<CommonState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::wrappers::IntervalStream::new(
        tokio::time::interval(Duration::from_secs(3)),
    )
    .then(move |_| {
        let state = state.clone();

        async move {
            let height = state.chain_service.find_latest_height().await;
            Ok(Event::default().data(height.to_string()))
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
