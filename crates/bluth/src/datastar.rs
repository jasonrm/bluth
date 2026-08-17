#[cfg(feature = "axum")]
use axum::http::{StatusCode, header};
#[cfg(feature = "axum")]
use axum::response::{IntoResponse, Response};
use std::fmt::{Display, Write};
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
            let mut lines = ElementLineWriter {
                f,
                line_open: false,
                pending_cr: false,
            };
            write!(lines, "{}", element)?;
            lines.finish()?;
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

        let map = crate::signal::SignalMap::merge(&self.signals);
        writeln!(f, "data: signals {}", map)?;

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

pub struct OnInterval {
    pub duration: Duration,
    pub leading: bool,
    pub view_transition: bool,
}

impl OnInterval {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            leading: false,
            view_transition: false,
        }
    }
}

struct ElementLineWriter<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    line_open: bool,
    pending_cr: bool,
}

impl ElementLineWriter<'_, '_> {
    fn write_visible(&mut self, ch: char) -> std::fmt::Result {
        if !self.line_open {
            write!(self.f, "data: elements ")?;
            self.line_open = true;
        }
        self.f.write_char(ch)
    }

    fn end_line(&mut self) -> std::fmt::Result {
        if !self.line_open {
            write!(self.f, "data: elements ")?;
        }
        writeln!(self.f)?;
        self.line_open = false;
        Ok(())
    }

    fn finish(mut self) -> std::fmt::Result {
        if self.pending_cr {
            self.write_visible('\r')?;
        }
        if self.line_open {
            writeln!(self.f)?;
        }
        Ok(())
    }
}

impl Write for ElementLineWriter<'_, '_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for ch in s.chars() {
            if self.pending_cr {
                self.pending_cr = false;
                if ch == '\n' {
                    self.end_line()?;
                    continue;
                }
                self.write_visible('\r')?;
            }
            if ch == '\n' {
                self.end_line()?;
            } else if ch == '\r' {
                self.pending_cr = true;
            } else {
                self.write_visible(ch)?;
            }
        }
        Ok(())
    }
}

impl Display for OnInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ms = self.duration.as_millis();
        write!(f, "data-on-interval__duration.")?;
        if ms >= 1000 && ms % 1000 == 0 {
            write!(f, "{}s", ms / 1000)?;
        } else {
            write!(f, "{}ms", ms)?;
        }
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
    fn patch_elements_splits_multiline_display() {
        let patch = PatchElements::new(vec!["<div>\n<span>hi</span>\n</div>"]);
        let sse = patch.to_string();
        assert!(sse.contains("data: elements <div>\n"));
        assert!(sse.contains("data: elements <span>hi</span>\n"));
        assert!(sse.contains("data: elements </div>\n"));
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
        let interval = OnInterval {
            duration: Duration::from_secs(1),
            leading: false,
            view_transition: false,
        };
        assert_eq!(interval.to_string(), "data-on-interval__duration.1s");
    }

    #[test]
    fn test_datastar_interval_milliseconds() {
        let interval = OnInterval {
            duration: Duration::from_millis(500),
            leading: false,
            view_transition: false,
        };
        assert_eq!(interval.to_string(), "data-on-interval__duration.500ms");
    }

    #[test]
    fn test_datastar_interval_minutes() {
        let interval = OnInterval {
            duration: Duration::from_secs(120),
            leading: false,
            view_transition: false,
        };
        assert_eq!(interval.to_string(), "data-on-interval__duration.120s");
    }

    #[test]
    fn test_datastar_interval_with_leading() {
        let interval = OnInterval {
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
        let interval = OnInterval {
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
        let interval = OnInterval {
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
        let interval = OnInterval {
            duration: Duration::from_millis(1500),
            leading: false,
            view_transition: false,
        };
        assert_eq!(interval.to_string(), "data-on-interval__duration.1500ms");
    }
}
