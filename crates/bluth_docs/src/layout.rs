use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bluth::Document;
use std::fmt::Display;

pub struct HtmlResponse<T: Display>(pub Document<T>);

impl<T: Display> IntoResponse for HtmlResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::response::Html(self.0.to_string())).into_response()
    }
}
