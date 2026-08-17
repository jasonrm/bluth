#[cfg(feature = "axum")]
use axum::http::{StatusCode, header};
#[cfg(feature = "axum")]
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
    pub mode: PatchMode,
    pub namespace: Option<PatchNamespace>,
    pub view_transition: bool,
    pub elements: Vec<T>,
}

impl<T> PatchElements<T>
where
    T: Display,
{
    pub fn new(elements: Vec<T>) -> Self {
        Self {
            selector: None,
            mode: PatchMode::Outer,
            namespace: None,
            view_transition: false,
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

        if self.mode != PatchMode::Outer {
            writeln!(f, "data: mode {}", self.mode.as_ref())?;
        }

        if let Some(namespace) = self.namespace {
            writeln!(f, "data: namespace {}", namespace.as_ref())?;
        }

        if self.view_transition {
            writeln!(f, "data: useViewTransition {}", self.view_transition)?;
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

#[cfg(feature = "axum")]
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
    pub only_if_missing: bool,
    pub signals: Vec<T>,
}

impl<T: SignalEnum> PatchSignals<T> {
    pub fn new(signals: Vec<T>) -> Self {
        Self {
            only_if_missing: false,
            signals,
        }
    }
}

impl<T: SignalEnum> Display for PatchSignals<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "event: datastar-patch-signals")?;

        if self.only_if_missing {
            writeln!(f, "data: onlyIfMissing {}", self.only_if_missing)?;
        }

        let merged = crate::signal::merge_signals(&self.signals);
        writeln!(f, "data: signals {}", merged)?;

        writeln!(f)?;

        Ok(())
    }
}

#[cfg(feature = "axum")]
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
    pub duration: Duration,
    pub leading: bool,
    pub view_transition: bool,
}

impl DatastarInterval {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            leading: false,
            view_transition: false,
        }
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
    use crate::Signal;

    #[test]
    fn default_patch_elements_omits_mode_and_view_transition() {
        let patch = PatchElements::new(vec!["<span>hi</span>"]);
        let sse = patch.to_string();
        assert!(sse.starts_with("event: datastar-patch-elements\n"));
        assert!(sse.contains("data: elements <span>hi</span>\n"));
        assert!(!sse.contains("data: mode"));
        assert!(!sse.contains("data: useViewTransition"));
        assert!(!sse.contains("data: selector"));
    }

    #[test]
    fn inner_patch_elements_emits_selector_and_mode() {
        let patch = PatchElements {
            selector: Some("#ticker-text".into()),
            mode: PatchMode::Inner,
            ..PatchElements::new(vec!["Hello World"])
        };
        let sse = patch.to_string();
        assert!(sse.contains("data: selector #ticker-text\n"));
        assert!(sse.contains("data: mode inner\n"));
        assert!(sse.contains("data: elements Hello World\n"));
        assert!(!sse.contains("data: useViewTransition"));
    }

    #[test]
    fn default_patch_signals_omits_only_if_missing() {
        #[derive(Signal)]
        enum CountSignals {
            Count(i32),
        }

        let patch = PatchSignals::new(vec![CountSignals::Count(1)]);
        let sse = patch.to_string();
        assert!(sse.starts_with("event: datastar-patch-signals\n"));
        assert!(sse.contains("data: signals {\"count\":1}\n"));
        assert!(!sse.contains("data: onlyIfMissing"));
    }

    #[test]
    fn patch_signals_emits_only_if_missing_when_set() {
        #[derive(Signal)]
        enum CountSignals {
            Count(i32),
        }

        let patch = PatchSignals {
            only_if_missing: true,
            ..PatchSignals::new(vec![CountSignals::Count(1)])
        };
        let sse = patch.to_string();
        assert!(sse.contains("data: onlyIfMissing true\n"));
        assert!(sse.contains("data: signals {\"count\":1}\n"));
    }

    #[test]
    fn test_datastar_interval_seconds() {
        let interval = DatastarInterval {
            duration: Duration::from_secs(1),
            leading: false,
            view_transition: false,
        };
        assert_eq!(interval.to_string(), "data-on-interval__duration.1s");
    }

    #[test]
    fn test_datastar_interval_milliseconds() {
        let interval = DatastarInterval {
            duration: Duration::from_millis(500),
            leading: false,
            view_transition: false,
        };
        assert_eq!(interval.to_string(), "data-on-interval__duration.500ms");
    }

    #[test]
    fn test_datastar_interval_minutes() {
        let interval = DatastarInterval {
            duration: Duration::from_secs(120),
            leading: false,
            view_transition: false,
        };
        assert_eq!(interval.to_string(), "data-on-interval__duration.120s");
    }

    #[test]
    fn test_datastar_interval_with_leading() {
        let interval = DatastarInterval {
            duration: Duration::from_secs(1),
            leading: true,
            view_transition: false,
        };
        assert_eq!(
            interval.to_string(),
            "data-on-interval__duration.1s.leading"
        );
    }

    #[test]
    fn test_datastar_interval_with_viewtransition() {
        let interval = DatastarInterval {
            duration: Duration::from_millis(500),
            leading: false,
            view_transition: true,
        };
        assert_eq!(
            interval.to_string(),
            "data-on-interval__duration.500ms__viewtransition"
        );
    }

    #[test]
    fn test_datastar_interval_with_all_modifiers() {
        let interval = DatastarInterval {
            duration: Duration::from_secs(2),
            leading: true,
            view_transition: true,
        };
        assert_eq!(
            interval.to_string(),
            "data-on-interval__duration.2s.leading__viewtransition"
        );
    }

    #[test]
    fn test_datastar_interval_mixed_units() {
        let interval = DatastarInterval {
            duration: Duration::from_millis(1500),
            leading: false,
            view_transition: false,
        };
        assert_eq!(interval.to_string(), "data-on-interval__duration.1500ms");
    }
}
