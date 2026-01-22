use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use std::fmt::Display;
use std::time::Duration;
use strum::AsRefStr;

use crate::signal::SignalEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum PatchMode {
    Outer,
    Inner,
    Replace,
    Prepend,
    Append,
    Before,
    After,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
pub enum PatchNamespace {
    Svg,
    MathML,
}

pub struct PatchElements<T> {
    pub selector: Option<String>,
    pub mode: Option<PatchMode>,
    pub namespace: Option<PatchNamespace>,
    pub use_view_transition: Option<bool>,
    pub elements: Vec<T>,
}

impl<T> PatchElements<T>
where
    T: Display,
{
    pub fn new(elements: Vec<T>) -> Self {
        Self {
            selector: None,
            mode: None,
            namespace: None,
            use_view_transition: None,
            elements,
        }
    }
}

impl<T> Display for PatchElements<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "event: datastar-patch-elements")?;

        if let Some(ref selector) = self.selector {
            writeln!(f, "data: selector {}", selector)?;
        }

        if let Some(mode) = self.mode {
            writeln!(f, "data: mode {}", mode.as_ref())?;
        }

        if let Some(namespace) = self.namespace {
            writeln!(f, "data: namespace {}", namespace.as_ref())?;
        }

        if let Some(use_view_transition) = self.use_view_transition {
            writeln!(f, "data: useViewTransition {}", use_view_transition)?;
        }

        for element in &self.elements {
            for line in element.to_string().lines() {
                writeln!(f, "data: elements {}", line)?;
            }
        }

        writeln!(f)?;

        Ok(())
    }
}

impl<T> IntoResponse for PatchElements<T>
where
    T: Display,
{
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/event-stream")],
            self.to_string(),
        )
            .into_response()
    }
}

pub struct PatchSignals<T: SignalEnum> {
    pub only_if_missing: Option<bool>,
    pub signals: Vec<T>,
}

impl<T: SignalEnum> PatchSignals<T> {
    pub fn new(signals: Vec<T>) -> Self {
        Self {
            only_if_missing: None,
            signals,
        }
    }

    pub fn only_if_missing(mut self, value: bool) -> Self {
        self.only_if_missing = Some(value);
        self
    }
}

impl<T: SignalEnum> Display for PatchSignals<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "event: datastar-patch-signals")?;

        if let Some(only_if_missing) = self.only_if_missing {
            writeln!(f, "data: onlyIfMissing {}", only_if_missing)?;
        }

        let merged = crate::signal::merge_signals(&self.signals);
        writeln!(f, "data: signals {}", merged)?;

        writeln!(f)?;

        Ok(())
    }
}

impl<T: SignalEnum> IntoResponse for PatchSignals<T> {
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/event-stream")],
            self.to_string(),
        )
            .into_response()
    }
}

pub struct DatastarInterval {
    duration: Duration,
    leading: bool,
    view_transition: bool,
}

impl DatastarInterval {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            leading: false,
            view_transition: false,
        }
    }

    pub fn leading(mut self) -> Self {
        self.leading = true;
        self
    }

    pub fn viewtransition(mut self) -> Self {
        self.view_transition = true;
        self
    }
}

impl Display for DatastarInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ms = self.duration.as_millis();
        let duration_str = if ms >= 1000 && ms % 1000 == 0 {
            format!("{}s", ms / 1000)
        } else {
            format!("{}ms", ms)
        };

        write!(f, "data-on-interval__duration.{}", duration_str)?;
        if self.leading {
            write!(f, ".leading")?;
        }
        if self.view_transition {
            write!(f, "__viewtransition")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datastar_interval_seconds() {
        let interval = DatastarInterval::new(Duration::from_secs(1));
        assert_eq!(interval.to_string(), "data-on-interval__duration.1s");
    }

    #[test]
    fn test_datastar_interval_milliseconds() {
        let interval = DatastarInterval::new(Duration::from_millis(500));
        assert_eq!(interval.to_string(), "data-on-interval__duration.500ms");
    }

    #[test]
    fn test_datastar_interval_minutes() {
        let interval = DatastarInterval::new(Duration::from_secs(120));
        assert_eq!(interval.to_string(), "data-on-interval__duration.120s");
    }

    #[test]
    fn test_datastar_interval_with_leading() {
        let interval = DatastarInterval::new(Duration::from_secs(1)).leading();
        assert_eq!(
            interval.to_string(),
            "data-on-interval__duration.1s.leading"
        );
    }

    #[test]
    fn test_datastar_interval_with_viewtransition() {
        let interval = DatastarInterval::new(Duration::from_millis(500)).viewtransition();
        assert_eq!(
            interval.to_string(),
            "data-on-interval__duration.500ms__viewtransition"
        );
    }

    #[test]
    fn test_datastar_interval_with_all_modifiers() {
        let interval = DatastarInterval::new(Duration::from_secs(2))
            .leading()
            .viewtransition();
        assert_eq!(
            interval.to_string(),
            "data-on-interval__duration.2s.leading__viewtransition"
        );
    }

    #[test]
    fn test_datastar_interval_mixed_units() {
        let interval = DatastarInterval::new(Duration::from_millis(1500));
        assert_eq!(interval.to_string(), "data-on-interval__duration.1500ms");
    }
}
