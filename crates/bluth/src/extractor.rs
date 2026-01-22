use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;

use crate::signal::SignalSelector;

pub struct Signal<S: SignalSelector>(pub S::Value);

pub struct Signals<T>(pub T);

#[derive(Debug)]
pub enum SignalRejection {
    MissingDatastarHeader,
    InvalidJson(String),
    MissingSignal(&'static str),
}

impl IntoResponse for SignalRejection {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            SignalRejection::MissingDatastarHeader => {
                (StatusCode::BAD_REQUEST, "Missing Datastar-Request header")
            }
            SignalRejection::InvalidJson(ref err) => (
                StatusCode::BAD_REQUEST,
                Box::leak(format!("Invalid JSON: {}", err).into_boxed_str()) as &str,
            ),
            SignalRejection::MissingSignal(signal) => (
                StatusCode::BAD_REQUEST,
                Box::leak(format!("Missing signal: {}", signal).into_boxed_str()) as &str,
            ),
        };
        (status, message).into_response()
    }
}

pub trait FromSignalMap: Sized {
    fn from_signal_map(
        signals: &HashMap<String, serde_json::Value>,
    ) -> Result<Self, SignalRejection>;
}

impl<S> FromSignalMap for Signal<S>
where
    S: SignalSelector,
{
    fn from_signal_map(
        signals: &HashMap<String, serde_json::Value>,
    ) -> Result<Self, SignalRejection> {
        let value = signals
            .get(S::NAME)
            .ok_or(SignalRejection::MissingSignal(S::NAME))?;

        let parsed: S::Value = serde_json::from_value(value.clone())
            .map_err(|e| SignalRejection::InvalidJson(e.to_string()))?;

        Ok(Signal(parsed))
    }
}

impl<S1, S2> FromSignalMap for (Signal<S1>, Signal<S2>)
where
    S1: SignalSelector,
    S2: SignalSelector,
{
    fn from_signal_map(
        signals: &HashMap<String, serde_json::Value>,
    ) -> Result<Self, SignalRejection> {
        Ok((
            Signal::<S1>::from_signal_map(signals)?,
            Signal::<S2>::from_signal_map(signals)?,
        ))
    }
}

impl<S1, S2, S3> FromSignalMap for (Signal<S1>, Signal<S2>, Signal<S3>)
where
    S1: SignalSelector,
    S2: SignalSelector,
    S3: SignalSelector,
{
    fn from_signal_map(
        signals: &HashMap<String, serde_json::Value>,
    ) -> Result<Self, SignalRejection> {
        Ok((
            Signal::<S1>::from_signal_map(signals)?,
            Signal::<S2>::from_signal_map(signals)?,
            Signal::<S3>::from_signal_map(signals)?,
        ))
    }
}

impl<S1, S2, S3, S4> FromSignalMap for (Signal<S1>, Signal<S2>, Signal<S3>, Signal<S4>)
where
    S1: SignalSelector,
    S2: SignalSelector,
    S3: SignalSelector,
    S4: SignalSelector,
{
    fn from_signal_map(
        signals: &HashMap<String, serde_json::Value>,
    ) -> Result<Self, SignalRejection> {
        Ok((
            Signal::<S1>::from_signal_map(signals)?,
            Signal::<S2>::from_signal_map(signals)?,
            Signal::<S3>::from_signal_map(signals)?,
            Signal::<S4>::from_signal_map(signals)?,
        ))
    }
}

async fn parse_signals_from_request(
    req: Request,
) -> Result<HashMap<String, serde_json::Value>, SignalRejection> {
    let (parts, body) = req.into_parts();

    let datastar_request = parts
        .headers
        .get("Datastar-Request")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("false");

    if datastar_request != "true" {
        return Err(SignalRejection::MissingDatastarHeader);
    }

    let content_type = parts
        .headers
        .get("Content-Type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.contains("application/json") {
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|e| SignalRejection::InvalidJson(e.to_string()))?;

        serde_json::from_slice(&body_bytes).map_err(|e| SignalRejection::InvalidJson(e.to_string()))
    } else {
        let query_string = parts.uri.query().unwrap_or("");
        let mut datastar_json = None;

        for pair in query_string.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == "datastar" {
                    let decoded = urlencoding::decode(value)
                        .map_err(|e| SignalRejection::InvalidJson(e.to_string()))?;
                    datastar_json = Some(decoded.into_owned());
                    break;
                }
            }
        }

        let json_str = datastar_json.ok_or_else(|| {
            SignalRejection::InvalidJson("Missing datastar query parameter".to_string())
        })?;

        serde_json::from_str(&json_str).map_err(|e| SignalRejection::InvalidJson(e.to_string()))
    }
}

impl<S, T> FromRequest<S> for Signal<T>
where
    S: Send + Sync,
    T: SignalSelector,
{
    type Rejection = SignalRejection;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let signals = parse_signals_from_request(req).await?;
        Signal::<T>::from_signal_map(&signals)
    }
}

impl<S, T> FromRequest<S> for Signals<T>
where
    S: Send + Sync,
    T: FromSignalMap,
{
    type Rejection = SignalRejection;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let signals = parse_signals_from_request(req).await?;
        Ok(Signals(T::from_signal_map(&signals)?))
    }
}
