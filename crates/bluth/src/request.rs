use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::signal::{SignalMap, SignalName};

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
            .unwrap_or("")
            .contains("application/json")
    }

    pub async fn bytes(self) -> Result<axum::body::Bytes, String> {
        axum::body::to_bytes(self.body, usize::MAX)
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
pub enum SignalRejection {
    MissingDatastarHeader,
    InvalidJson(String),
    MissingSignal(&'static str),
}

impl IntoResponse for SignalRejection {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            SignalRejection::MissingDatastarHeader => (
                StatusCode::BAD_REQUEST,
                "Missing Datastar-Request header".to_owned(),
            ),
            SignalRejection::InvalidJson(err) => {
                (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", err))
            }
            SignalRejection::MissingSignal(signal) => (
                StatusCode::BAD_REQUEST,
                format!("Missing signal: {}", signal),
            ),
        };
        (status, message).into_response()
    }
}

impl SignalMap {
    pub fn from_json(bytes: &[u8]) -> Result<Self, SignalRejection> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|e| SignalRejection::InvalidJson(e.to_string()))?;
        Self::from_value(value)
    }

    pub fn from_query(query: &str) -> Result<Self, SignalRejection> {
        let encoded = query.split('&').find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            (key == "datastar").then_some(value)
        });
        let encoded = encoded.ok_or_else(|| {
            SignalRejection::InvalidJson("Missing datastar query parameter".to_string())
        })?;
        let json = urlencoding::decode(encoded)
            .map_err(|e| SignalRejection::InvalidJson(e.to_string()))?;
        Self::from_str(&json)
    }

    pub fn from_str(json: &str) -> Result<Self, SignalRejection> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| SignalRejection::InvalidJson(e.to_string()))?;
        Self::from_value(value)
    }

    fn from_value(value: serde_json::Value) -> Result<Self, SignalRejection> {
        match value {
            serde_json::Value::Object(values) => Ok(Self { values }),
            _ => Err(SignalRejection::InvalidJson(
                "expected a JSON object".to_string(),
            )),
        }
    }

    pub fn signal<S: SignalName>(&self) -> Result<S::Value, SignalRejection> {
        let value = self
            .values
            .get(S::NAME)
            .ok_or(SignalRejection::MissingSignal(S::NAME))?;
        serde_json::from_value(value.clone())
            .map_err(|e| SignalRejection::InvalidJson(e.to_string()))
    }
}

impl<S, T> FromRequest<S> for Signal<T>
where
    S: Send + Sync,
    T: SignalName,
{
    type Rejection = SignalRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let map = SignalMap::from_request(req, state).await?;
        Ok(Signal(map.signal::<T>()?))
    }
}

impl<S> FromRequest<S> for SignalMap
where
    S: Send + Sync,
{
    type Rejection = SignalRejection;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let request = DatastarRequest::from_http(req);
        if request.header("Datastar-Request") != Some("true") {
            return Err(SignalRejection::MissingDatastarHeader);
        }
        if request.json() {
            let bytes = request
                .bytes()
                .await
                .map_err(SignalRejection::InvalidJson)?;
            return SignalMap::from_json(&bytes);
        }
        let query = request.query().unwrap_or("").to_owned();
        SignalMap::from_query(&query)
    }
}
