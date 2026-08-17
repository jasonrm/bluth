pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/assets/generated.rs"));
}

pub struct Asset {
    pub name: &'static str,
    pub bytes: &'static [u8],
    pub url: &'static str,
    pub etag: &'static str,
    pub content_type: &'static str,
}

impl Asset {
    pub fn datastar() -> Self {
        Self {
            name: "datastar",
            bytes: generated::DATASTAR,
            url: generated::DATASTAR_URL,
            etag: generated::DATASTAR_ETAG,
            content_type: "application/javascript",
        }
    }

    pub fn styles() -> Self {
        Self {
            name: "styles",
            bytes: generated::STYLES,
            url: generated::STYLES_URL,
            etag: generated::STYLES_ETAG,
            content_type: "text/css",
        }
    }

    pub fn from_path(name: &str) -> Option<Self> {
        let base = Self::base(name)?;
        [Self::datastar(), Self::styles()]
            .into_iter()
            .find(|asset| asset.name == base)
    }

    fn base(name: &str) -> Option<&str> {
        let (base, ext) = name.rsplit_once('.')?;
        if ext != "css" && ext != "js" {
            return None;
        }
        let (stem, _hash) = base.rsplit_once('.')?;
        Some(stem)
    }

    pub fn response(&self, headers: &axum::http::HeaderMap) -> axum::response::Response {
        use axum::http::{StatusCode, header};
        use axum::response::IntoResponse;

        let not_modified = headers
            .get(header::IF_NONE_MATCH)
            .and_then(|value| value.to_str().ok())
            == Some(self.etag);

        if not_modified {
            return (
                StatusCode::NOT_MODIFIED,
                [
                    (header::ETAG, self.etag),
                    (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                ],
                "",
            )
                .into_response();
        }

        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, self.content_type),
                (header::ETAG, self.etag),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            self.bytes,
        )
            .into_response()
    }

    pub async fn get(
        axum::extract::Path(name): axum::extract::Path<String>,
        headers: axum::http::HeaderMap,
    ) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;

        match Self::from_path(&name) {
            Some(asset) => asset.response(&headers),
            None => (StatusCode::NOT_FOUND, "not found").into_response(),
        }
    }
}
