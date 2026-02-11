use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use bluth::datastar::{PatchElements, PatchMode};
use bytes::Bytes;
use http_body_util::StreamBody;
use hyper::body::Frame;
use std::convert::Infallible;
use std::time::Duration;

pub async fn sse_ticker() -> Response {
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

            let event_data = patch.to_string();
            yield Ok::<_, Infallible>(Frame::data(Bytes::from(event_data)));
        }
    };

    let body = StreamBody::new(stream);
    let body = Body::new(body);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(body)
        .unwrap()
}
