use crate::{Error, Signal, SignalMap, SignalName};
use axum::{
    extract::FromRequest,
    http::{StatusCode, header},
    response::IntoResponse,
};
use bluth_macros::Element;

#[tokio::test]
async fn signal_extractor_post_json() -> Result<(), anyhow::Error> {
    #[derive(Signal)]
    enum SearchSignals {
        SearchTerm(String),
    }

    use axum::{
        body::Body,
        extract::Request,
        http::{Method, header},
    };

    let json_body = r#"{"searchTerm":"test query"}"#;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/search")
        .header(header::CONTENT_TYPE, "application/json")
        .header("Datastar-Request", "true")
        .body(Body::from(json_body))?;

    let result: Result<Signal<SearchTerm>, _> =
        Signal::<SearchTerm>::from_request(request, &()).await;

    let Signal(search_term) = result.expect("Failed to extract signal");

    assert_eq!(search_term, "test query");
    assert_eq!(SearchTerm::NAME, "searchTerm");

    Ok(())
}

#[tokio::test]
async fn signal_extractor_get_query() -> Result<(), anyhow::Error> {
    #[derive(Signal)]
    enum SearchSignals {
        SearchTerm(String),
    }

    use axum::{body::Body, extract::Request, http::Method};

    let query = "datastar=%7B%22searchTerm%22%3A%22test%20query%22%7D";

    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/search?{}", query))
        .header("Datastar-Request", "true")
        .body(Body::empty())?;

    let result: Result<Signal<SearchTerm>, _> =
        Signal::<SearchTerm>::from_request(request, &()).await;

    let Signal(search_term) = result.expect("Failed to extract signal");

    assert_eq!(search_term, "test query");

    Ok(())
}

#[tokio::test]
async fn signal_extractor_multiple_signals() -> Result<(), anyhow::Error> {
    #[derive(Signal)]
    enum RegisterSignals {
        UserName(String),
        UserEmail(String),
    }

    use axum::{body::Body, extract::Request, http::Method};

    let json_body = r#"{"userName":"John Doe","userEmail":"john@example.com"}"#;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/register")
        .header(header::CONTENT_TYPE, "application/json")
        .header("Datastar-Request", "true")
        .body(Body::from(json_body))?;

    let result: Result<SignalMap, _> = SignalMap::from_request(request, &()).await;

    let map = result.expect("Failed to extract signals");

    assert_eq!(map.signal::<UserName>().expect("userName"), "John Doe");
    assert_eq!(
        map.signal::<UserEmail>().expect("userEmail"),
        "john@example.com"
    );
    assert_eq!(UserName::NAME, "userName");
    assert_eq!(UserEmail::NAME, "userEmail");

    Ok(())
}

#[tokio::test]
async fn signal_extractor_missing_header() -> Result<(), anyhow::Error> {
    #[derive(Signal)]
    enum SearchSignals {
        SearchTerm(String),
    }

    use axum::{body::Body, extract::Request, http::Method};

    let json_body = r#"{"searchTerm":"test"}"#;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json_body))?;

    let result: Result<Signal<SearchTerm>, _> =
        Signal::<SearchTerm>::from_request(request, &()).await;

    let err = result.expect_err("header required");
    assert!(matches!(err, Error::MissingDatastarHeader));

    Ok(())
}

#[tokio::test]
async fn signal_extractor_missing_query() -> Result<(), anyhow::Error> {
    use axum::{body::Body, extract::Request, http::Method};

    let request = Request::builder()
        .method(Method::GET)
        .uri("/search")
        .header("Datastar-Request", "true")
        .body(Body::empty())?;

    let result: Result<SignalMap, _> = SignalMap::from_request(request, &()).await;

    let err = result.expect_err("datastar query required");
    assert!(matches!(err, Error::MissingDatastarQuery));

    Ok(())
}

#[tokio::test]
async fn signal_extractor_invalid_json() -> Result<(), anyhow::Error> {
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, header},
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri("/search")
        .header(header::CONTENT_TYPE, "application/json")
        .header("Datastar-Request", "true")
        .body(Body::from("not-json"))?;

    let result: Result<SignalMap, _> = SignalMap::from_request(request, &()).await;

    let err = result.expect_err("json required");
    assert!(matches!(err, Error::Json(_)));

    Ok(())
}

#[tokio::test]
async fn signal_extractor_missing_signal() -> Result<(), anyhow::Error> {
    #[derive(Signal)]
    enum SearchSignals {
        SearchTerm(String),
    }

    use axum::{body::Body, extract::Request, http::Method};

    let json_body = r#"{"otherSignal":"value"}"#;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/search")
        .header(header::CONTENT_TYPE, "application/json")
        .header("Datastar-Request", "true")
        .body(Body::from(json_body))?;

    let result: Result<Signal<SearchTerm>, _> =
        Signal::<SearchTerm>::from_request(request, &()).await;

    let err = result.expect_err("signal required");
    assert!(matches!(err, Error::MissingSignal("searchTerm")));

    Ok(())
}

#[tokio::test]
async fn enum_to_html() -> Result<(), anyhow::Error> {
    #[derive(Element)]
    #[element("span")]
    enum Hello {
        World(String),
    }

    async fn with_status_and_array_headers() -> Result<impl IntoResponse, anyhow::Error> {
        let hello = Hello::World("world".into());

        Ok((
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            hello.to_string(),
        ))
    }

    let (parts, body) = with_status_and_array_headers()
        .await?
        .into_response()
        .into_parts();

    assert_eq!(parts.status, StatusCode::NOT_FOUND);

    let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
    let body_str = String::from_utf8(body_bytes.to_vec())?;

    assert_eq!(body_str, "<span>world</span>");

    Ok(())
}
