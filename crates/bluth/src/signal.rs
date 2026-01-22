use std::collections::HashMap;

pub trait SignalEnum: Sized + serde::Serialize {
    fn signal_name(&self) -> &'static str;
    fn to_json_value(&self) -> serde_json::Value;
}

pub trait SignalSelector: Sized {
    type Value: for<'de> serde::Deserialize<'de>;
    type Enum: SignalEnum;

    const NAME: &'static str;

    fn extract(value: &Self::Enum) -> Option<&Self::Value>;
    fn into_inner(value: Self::Enum) -> Option<Self::Value>;
    fn wrap(value: Self::Value) -> Self::Enum;
}

pub struct SignalValue<S: SignalSelector>(pub S::Value);

impl<S: SignalSelector> SignalValue<S> {
    pub fn new(value: S::Value) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> S::Value {
        self.0
    }

    pub fn into_enum(self) -> S::Enum {
        S::wrap(self.0)
    }
}

impl<S: SignalSelector> std::ops::Deref for SignalValue<S> {
    type Target = S::Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: SignalSelector> std::ops::DerefMut for SignalValue<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: SignalSelector> Clone for SignalValue<S>
where
    S::Value: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: SignalSelector> std::fmt::Debug for SignalValue<S>
where
    S::Value: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SignalValue").field(&self.0).finish()
    }
}

pub trait OptDisplay {
    fn opt_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<T: std::fmt::Display> OptDisplay for Option<T> {
    fn opt_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self { v.fmt(f) } else { Ok(()) }
    }
}

impl<S: SignalSelector> std::fmt::Display for SignalValue<S>
where
    S::Value: OptDisplay,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.opt_fmt(f)
    }
}

impl<S: SignalSelector> PartialEq for SignalValue<S>
where
    S::Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<S: SignalSelector> Eq for SignalValue<S> where S::Value: Eq {}

pub fn merge_signals<T: SignalEnum>(signals: &[T]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for signal in signals {
        map.insert(signal.signal_name().to_string(), signal.to_json_value());
    }
    serde_json::Value::Object(map)
}

pub fn signals_from_map<S: SignalSelector>(
    signals: &HashMap<String, serde_json::Value>,
) -> Option<S::Value> {
    signals
        .get(S::NAME)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}
