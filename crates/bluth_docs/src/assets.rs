pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/assets/generated.rs"));
}

fn base_name(name: &str) -> Option<&str> {
    if let Some((base, ext)) = name.rsplit_once('.') {
        if ext == "css" || ext == "js" {
            if let Some((b, _hash)) = base.rsplit_once('.') {
                return Some(b);
            }
        }
    }
    None
}

pub fn asset_url(name: &str) -> Option<&'static str> {
    match name {
        "datastar" => Some(generated::DATASTAR_URL),
        "styles" => Some(generated::STYLES_URL),
        _ => None,
    }
}

pub fn asset_bytes(name: &str) -> Option<&'static [u8]> {
    let base = base_name(name)?;
    match base {
        "datastar" => Some(generated::DATASTAR),
        "styles" => Some(generated::STYLES),
        _ => None,
    }
}

pub fn asset_etag(name: &str) -> Option<&'static str> {
    let base = base_name(name)?;
    match base {
        "datastar" => Some(generated::DATASTAR_ETAG),
        "styles" => Some(generated::STYLES_ETAG),
        _ => None,
    }
}

pub fn asset_content_type(name: &str) -> Option<&'static str> {
    let base = base_name(name)?;
    match base {
        "datastar" => Some("application/javascript"),
        "styles" => Some("text/css"),
        _ => None,
    }
}

pub async fn serve(
    axum::extract::Path(name): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> impl axum::response::IntoResponse {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;

    if let (Some(content), Some(etag), Some(content_type)) = (
        asset_bytes(&name),
        asset_etag(&name),
        asset_content_type(&name),
    ) {
        if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
            if if_none_match.to_str().ok() == Some(etag) {
                return (
                    StatusCode::NOT_MODIFIED,
                    [
                        (header::ETAG, etag),
                        (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                    ],
                    "",
                )
                    .into_response();
            }
        }

        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, content_type),
                (header::ETAG, etag),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            content,
        )
            .into_response()
    } else {
        (StatusCode::NOT_FOUND, "not found").into_response()
    }
}
