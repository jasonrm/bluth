use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::error::Error;
use crate::signal::{SignalMap, SignalName};

#[derive(Debug)]
pub struct Signal<S: SignalName>(pub S::Value);

pub struct DatastarRequest {
    parts: axum::http::request::Parts,
    body: axum::body::Body,
}

impl DatastarRequest {
    pub fn from_http(req: Request) -> Self {
        let (parts, body) = req.into_parts();
        Self { parts, body }
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.parts.headers.get(name).and_then(|v| v.to_str().ok())
    }

    pub fn query(&self) -> Option<&str> {
        self.parts.uri.query()
    }

    pub fn json(&self) -> bool {
        self.header("Content-Type")
            .is_some_and(|value| value.contains("application/json"))
    }

    pub fn datastar(&self) -> Result<(), Error> {
        match self.header("Datastar-Request") {
            Some("true") => Ok(()),
            _ => Err(Error::MissingDatastarHeader),
        }
    }

    pub async fn bytes(self) -> Result<axum::body::Bytes, Error> {
        axum::body::to_bytes(self.body, usize::MAX)
            .await
            .map_err(|e| Error::Body(e.to_string()))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.to_string();
        (StatusCode::BAD_REQUEST, message).into_response()
    }
}

impl SignalMap {
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        let value = Self::value_from_slice(bytes)?;
        Self::from_object(value)
    }

    pub fn from_query(query: &str) -> Result<Self, Error> {
        let encoded = Self::datastar_param(query)?;
        let json = Self::decode(encoded)?;
        Self::from_str(&json)
    }

    pub fn from_str(json: &str) -> Result<Self, Error> {
        let value = Self::value_from_str(json)?;
        Self::from_object(value)
    }

    fn value_from_slice(bytes: &[u8]) -> Result<serde_json::Value, Error> {
        serde_json::from_slice(bytes).map_err(|e| Error::Json(e.to_string()))
    }

    fn value_from_str(json: &str) -> Result<serde_json::Value, Error> {
        serde_json::from_str(json).map_err(|e| Error::Json(e.to_string()))
    }

    fn datastar_param(query: &str) -> Result<&str, Error> {
        let encoded = query.split('&').find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            (key == "datastar").then_some(value)
        });
        encoded.ok_or(Error::MissingDatastarQuery)
    }

    fn decode(encoded: &str) -> Result<String, Error> {
        urlencoding::decode(encoded)
            .map(|cow| cow.into_owned())
            .map_err(|e| Error::Decode(e.to_string()))
    }

    fn from_object(value: serde_json::Value) -> Result<Self, Error> {
        match value {
            serde_json::Value::Object(values) => Ok(Self { values }),
            _ => Err(Error::Json("expected a JSON object".to_string())),
        }
    }

    pub fn signal<S: SignalName>(&self) -> Result<S::Value, Error> {
        let value = self.value(S::NAME)?;
        Self::deserialize(value)
    }

    fn value(&self, name: &'static str) -> Result<&serde_json::Value, Error> {
        self.values.get(name).ok_or(Error::MissingSignal(name))
    }

    fn deserialize<T>(value: &serde_json::Value) -> Result<T, Error>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_value(value.clone()).map_err(|e| Error::Json(e.to_string()))
    }
}

impl<S, T> FromRequest<S> for Signal<T>
where
    S: Send + Sync,
    T: SignalName,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let map = SignalMap::from_request(req, state).await?;
        let value = map.signal::<T>()?;
        Ok(Signal(value))
    }
}

impl<S> FromRequest<S> for SignalMap
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let request = DatastarRequest::from_http(req);
        let header = request.datastar();
        let json = request.json();
        let query = request.query().unwrap_or("").to_owned();
        let map = match header {
            Err(err) => Err(err),
            Ok(()) if json => match request.bytes().await {
                Ok(bytes) => SignalMap::from_json(&bytes),
                Err(err) => Err(err),
            },
            Ok(()) => SignalMap::from_query(&query),
        };
        map
    }
}
