use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::Response;
use bluth::Element;
use bluth::datastar::{PatchElements, PatchMode};
use bytes::Bytes;
use http_body_util::StreamBody;
use hyper::body::Frame;
use std::convert::Infallible;
use std::time::Duration;

#[derive(Element)]
struct TickerText {
    #[element]
    text: String,
}

pub struct Ticker;

impl Ticker {
    pub async fn stream() -> Response {
        let words = ["Hello", "World", "from", "Bluth"];

        let stream = async_stream::stream! {
            let mut text = String::new();

            for (index, word) in words.into_iter().enumerate() {
                if index > 0 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    text.push(' ');
                }
                text.push_str(word);

                let patch = PatchElements {
                    selector: Some("#ticker-text".into()),
                    mode: PatchMode::Inner,
                    ..PatchElements::new(vec![TickerText {
                        text: text.clone(),
                    }])
                };

                let bytes = Bytes::from(patch.to_string());
                yield Ok::<_, Infallible>(Frame::data(bytes));
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
}
