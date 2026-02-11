use axum::response::sse::{Event, KeepAlive, Sse};
use bluth::datastar::{PatchElements, PatchMode};
use futures::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;

pub async fn sse_ticker() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let words = ["Hello", "World", "from", "Bluth"];

    let stream = async_stream::stream! {
        let mut accumulated = String::new();

        for (index, word) in words.into_iter().enumerate() {
            if index > 0 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                accumulated.push(' ');
            }
            accumulated.push_str(word);

            let patch = PatchElements::new(vec![accumulated.clone()])
                .selector("#ticker-text")
                .mode(PatchMode::Inner);

            // PatchElements::to_string() generates the complete SSE event
            let event_string = patch.to_string();

            // Extract event type and data lines
            let mut lines = event_string.lines();
            let event_type = lines.next(); // "event: datastar-patch-elements"

            // Collect all data lines
            let data_lines: Vec<String> = lines
                .filter_map(|line| line.strip_prefix("data: ").map(|s| s.to_string()))
                .collect();

            // Build the event with proper structure
            let mut event = Event::default();
            if let Some(evt) = event_type {
                if let Some(evt_name) = evt.strip_prefix("event: ") {
                    event = event.event(evt_name);
                }
            }

            // Join all data lines and call data() once
            if !data_lines.is_empty() {
                event = event.data(data_lines.join("\n"));
            }

            yield Ok::<_, Infallible>(event);
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
