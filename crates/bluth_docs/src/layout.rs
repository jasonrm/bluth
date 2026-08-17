use crate::assets::Asset;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bluth::{Body, Document, Element, Head, Html, Link, Script};
use std::fmt::Display;

#[derive(Element)]
#[element("nav")]
#[attr(class = "mb-8 flex gap-4 text-gray-400")]
pub struct Nav {
    #[element]
    links: Vec<NavLink>,
}

#[derive(Element)]
#[element("a")]
#[attr(href = "{href}", class = "underline hover:text-white")]
struct NavLink {
    href: &'static str,
    #[element]
    text: &'static str,
}

impl Nav {
    pub fn site() -> Self {
        Self {
            links: vec![
                NavLink {
                    href: "/",
                    text: "Home",
                },
                NavLink {
                    href: "/catalog",
                    text: "Catalog",
                },
            ],
        }
    }
}

pub fn document<T: Display>(content: T, datastar: Asset, styles: Asset) -> Document<T> {
    Document::new(Html {
        lang: "en",
        head: Head {
            link: vec![Link {
                id: Some("stylesheet"),
                href: styles.url,
            }],
            script: vec![Script {
                src: datastar.url,
                async_: false,
                type_: "module",
            }],
        },
        body: Body {
            class: "bg-gray-950 text-gray-100 min-h-screen",
            children: vec![content],
        },
    })
}

pub struct HtmlResponse<T: Display>(pub Document<T>);

impl<T: Display> IntoResponse for HtmlResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::response::Html(self.0.to_string())).into_response()
    }
}
