pub trait SignalEnum: Sized + serde::Serialize {
    fn name(&self) -> &'static str;
    fn json(&self) -> serde_json::Value;
}

pub trait SignalName: Sized {
    type Value: for<'de> serde::Deserialize<'de>;
    type Enum: SignalEnum;

    const NAME: &'static str;

    fn value(signal: &Self::Enum) -> Option<&Self::Value>;
    fn owned(signal: Self::Enum) -> Option<Self::Value>;
    fn from_value(value: Self::Value) -> Self::Enum;
}

pub struct SignalValue<S: SignalName>(pub S::Value);

impl<S: SignalName> SignalValue<S> {
    pub fn new(value: S::Value) -> Self {
        Self(value)
    }

    pub fn signal(self) -> S::Enum {
        S::from_value(self.0)
    }
}

impl<S: SignalName> std::ops::Deref for SignalValue<S> {
    type Target = S::Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: SignalName> Clone for SignalValue<S>
where
    S::Value: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: SignalName> std::fmt::Debug for SignalValue<S>
where
    S::Value: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SignalValue").field(&self.0).finish()
    }
}

pub trait OptionalDisplay {
    fn optional(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<T: std::fmt::Display> OptionalDisplay for Option<T> {
    fn optional(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self { v.fmt(f) } else { Ok(()) }
    }
}

impl<S: SignalName> std::fmt::Display for SignalValue<S>
where
    S::Value: OptionalDisplay,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.optional(f)
    }
}

impl<S: SignalName> PartialEq for SignalValue<S>
where
    S::Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<S: SignalName> Eq for SignalValue<S> where S::Value: Eq {}

pub struct SignalMap {
    pub values: serde_json::Map<String, serde_json::Value>,
}

impl SignalMap {
    pub fn merge<T: SignalEnum>(signals: &[T]) -> Self {
        let mut values = serde_json::Map::new();
        for signal in signals {
            values.insert(signal.name().to_string(), signal.json());
        }
        Self { values }
    }
}

impl std::fmt::Display for SignalMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::Value::Object(self.values.clone()))
    }
}
