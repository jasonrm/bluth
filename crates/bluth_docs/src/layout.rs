use crate::assets::asset_url;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bluth::{Body, Document, Head, Html, Link, Script};
use std::fmt::Display;

pub fn head() -> Head {
    let script = Script {
        src: asset_url("datastar").unwrap(),
        async_: false,
        type_: "module",
    };

    let style = Link {
        id: Some("stylesheet"),
        href: asset_url("styles").unwrap(),
    };

    Head {
        link: vec![style],
        script: vec![script],
    }
}

pub fn page<T: Display>(content: T) -> Response {
    let html = Html {
        lang: "en",
        head: head(),
        body: Body {
            class: "bg-gray-950 text-gray-100 min-h-screen",
            children: vec![content],
        },
    };

    let document = Document::new(html);

    (StatusCode::OK, axum::response::Html(document.to_string())).into_response()
}
