use crate::{Element, Signal, SignalSelector, SignalValue};

#[derive(Signal)]
pub enum TestSignals {
    UserName(String),
    SearchTerm(Option<String>),
    #[signal(name = "pageNum")]
    PageNumber(i32),
}

#[test]
fn data_bind_with_signal_selector() {
    #[derive(Element)]
    struct Hello {
        user_name: String,

        #[element("input")]
        #[attr(
            data_bind = UserName,
            "data-on:keydown" = "@get(/hello)",
            value = "{user_name}"
        )]
        input: (),
    }

    let hello = Hello {
        user_name: "John Doe".to_string(),
        input: (),
    };

    let html = hello.to_string();

    assert_eq!(
        html,
        "<input data-bind=\"userName\" data-on:keydown=\"@get(/hello)\" value=\"John Doe\"/>"
    );
}

#[test]
fn data_bind_with_field_bound_signal() {
    #[derive(Element)]
    struct SearchBar {
        search_term: SignalValue<SearchTerm>,

        #[element("input")]
        #[attr(data_bind = search_term, id = "search-input")]
        input: (),
    }

    let search_bar = SearchBar {
        search_term: SignalValue::new(Some("hello".to_string())),
        input: (),
    };

    let html = search_bar.to_string();

    assert_eq!(
        html,
        "<input data-bind=\"searchTerm\" id=\"search-input\"/>"
    );
}

#[test]
fn data_bind_with_field_bound_custom_signal_name() {
    #[derive(Element)]
    struct PageNav {
        page_num: SignalValue<PageNumber>,

        #[element("span")]
        #[attr(data_bind = page_num)]
        display: (),
    }

    let nav = PageNav {
        page_num: SignalValue::new(42),
        display: (),
    };

    let html = nav.to_string();

    assert_eq!(html, "<span data-bind=\"pageNum\"></span>");
}

#[test]
fn data_bind_with_nested_element_signals() {
    #[derive(Element)]
    struct Inner {
        inner_signal: SignalValue<UserName>,

        #[element("input")]
        #[attr(data_bind = inner_signal)]
        input: (),
    }

    #[derive(Element)]
    struct Outer {
        outer_signal: SignalValue<SearchTerm>,

        #[element("div")]
        #[attr(data_bind = outer_signal)]
        wrapper: (),

        #[element]
        inner: Inner,
    }

    let outer = Outer {
        outer_signal: SignalValue::new(Some("query".to_string())),
        wrapper: (),
        inner: Inner {
            inner_signal: SignalValue::new("user".to_string()),
            input: (),
        },
    };

    let html = outer.to_string();

    assert_eq!(
        html,
        "<div data-bind=\"searchTerm\"></div><input data-bind=\"userName\"/>"
    );
}

#[test]
fn data_bind_with_string_literal() {
    #[derive(Element)]
    struct Hello {
        #[element("input")]
        #[attr(data_bind = "legacySignal")]
        input: (),
    }

    let hello = Hello { input: () };

    let html = hello.to_string();

    assert_eq!(html, "<input data-bind=\"legacySignal\"/>");
}

#[test]
fn data_text() {
    #[derive(Element)]
    struct Hello {
        #[element("span")]
        #[attr(data_text = "$userName")]
        output: (),
    }

    let hello = Hello { output: () };

    let html = hello.to_string();

    assert_eq!(html, "<span data-text=\"$userName\"></span>");
}

#[test]
fn data_computed_with_quotes() {
    #[derive(Element)]
    #[element("div")]
    #[attr("data-computed" = r#"msg = "Hello " + name"#)]
    struct Hello {
        #[element("span")]
        output: (),
    }

    let hello = Hello { output: () };

    let html = hello.to_string();

    assert_eq!(
        html,
        "<div data-computed=\"msg = &quot;Hello &quot; + name\"><span></span></div>"
    );
}

#[test]
fn data_computed_with_complex_js() {
    #[derive(Element)]
    #[element("div")]
    #[attr(
        "data-timestamp" = "{timestamp}",
        "data-computed" = r#"formattedTime = "UTC: " + new Intl.DateTimeFormat('en-US', { dateStyle: 'short' }).format(new Date(parseInt($el.dataset.timestamp)))"#
    )]
    struct DateTimeDisplay {
        timestamp: i64,

        #[element("p")]
        #[attr("data-text" = "formattedTime")]
        display: (),
    }

    let display = DateTimeDisplay {
        timestamp: 1234567890000,
        display: (),
    };

    let html = display.to_string();

    assert_eq!(
        html,
        "<div data-timestamp=\"1234567890000\" data-computed=\"formattedTime = &quot;UTC: &quot; + new Intl.DateTimeFormat('en-US', { dateStyle: 'short' }).format(new Date(parseInt($el.dataset.timestamp)))\"><p data-text=\"formattedTime\"></p></div>"
    );
}

#[test]
fn interpolated_value_with_special_chars() {
    #[derive(Element)]
    #[element("div")]
    #[attr("data-label" = "{label}")]
    struct Hello {
        label: String,
    }

    let hello = Hello {
        label: r#"Say "Hello" & <wave>"#.to_string(),
    };

    let html = hello.to_string();

    assert_eq!(
        html,
        "<div data-label=\"Say &quot;Hello&quot; &amp; &lt;wave&gt;\"></div>"
    );
}

#[test]
fn selector_has_correct_name() {
    assert_eq!(UserName::NAME, "userName");
    assert_eq!(SearchTerm::NAME, "searchTerm");
    assert_eq!(PageNumber::NAME, "pageNum");
}

#[test]
fn selector_as_ref_str() {
    assert_eq!(UserName.as_ref(), "userName");
    assert_eq!(SearchTerm.as_ref(), "searchTerm");
    assert_eq!(PageNumber.as_ref(), "pageNum");
}

#[test]
fn wrap_and_extract() {
    let signal = UserName::wrap("hello".to_string());
    assert!(matches!(&signal, TestSignals::UserName(s) if s == "hello"));

    let extracted = UserName::extract(&signal);
    assert_eq!(extracted, Some(&"hello".to_string()));

    let wrong_signal = PageNumber::wrap(42);
    assert_eq!(UserName::extract(&wrong_signal), None);
}

#[test]
fn into_inner() {
    let signal = SearchTerm::wrap(Some("query".to_string()));
    let inner = SearchTerm::into_inner(signal);
    assert_eq!(inner, Some(Some("query".to_string())));

    let wrong_signal = UserName::wrap("test".to_string());
    let inner = SearchTerm::into_inner(wrong_signal);
    assert_eq!(inner, None);
}

#[test]
fn signal_enum_signal_name() {
    use crate::SignalEnum;

    let signal = TestSignals::UserName("test".to_string());
    assert_eq!(signal.signal_name(), "userName");

    let signal = TestSignals::SearchTerm(None);
    assert_eq!(signal.signal_name(), "searchTerm");

    let signal = TestSignals::PageNumber(1);
    assert_eq!(signal.signal_name(), "pageNum");
}

#[test]
fn signal_enum_to_json_value() {
    use crate::SignalEnum;

    let signal = TestSignals::UserName("test".to_string());
    assert_eq!(signal.to_json_value(), serde_json::json!("test"));

    let signal = TestSignals::SearchTerm(Some("query".to_string()));
    assert_eq!(signal.to_json_value(), serde_json::json!("query"));

    let signal = TestSignals::SearchTerm(None);
    assert_eq!(signal.to_json_value(), serde_json::Value::Null);

    let signal = TestSignals::PageNumber(42);
    assert_eq!(signal.to_json_value(), serde_json::json!(42));
}

#[test]
fn signal_enum_serialize() {
    let signal = TestSignals::UserName("john".to_string());
    let json = serde_json::to_string(&signal).unwrap();
    assert_eq!(json, r#"{"userName":"john"}"#);

    let signal = TestSignals::PageNumber(5);
    let json = serde_json::to_string(&signal).unwrap();
    assert_eq!(json, r#"{"pageNum":5}"#);
}

#[test]
fn signal_enum_clone() {
    let signal = TestSignals::UserName("test".to_string());
    let cloned = signal.clone();
    assert!(matches!(cloned, TestSignals::UserName(s) if s == "test"));
}

#[test]
fn signal_enum_debug() {
    let signal = TestSignals::UserName("test".to_string());
    let debug_str = format!("{:?}", signal);
    assert_eq!(debug_str, r#"UserName("test")"#);
}

#[test]
fn map_or_with_option() {
    #[derive(Element)]
    #[map_or("NONE")]
    pub struct SearchTermDisplay(pub Option<String>);

    let with_content = SearchTermDisplay(Some("content".to_string()));
    let html_with_content = with_content.to_string();
    assert_eq!(html_with_content, "content");

    let without_content = SearchTermDisplay(None);
    let html_without_content = without_content.to_string();
    assert_eq!(html_without_content, "NONE");
}

#[test]
fn merge_signals() {
    let signals = vec![
        TestSignals::UserName("john".to_string()),
        TestSignals::PageNumber(3),
    ];

    let merged = crate::signal::merge_signals(&signals);

    assert_eq!(merged["userName"], serde_json::json!("john"));
    assert_eq!(merged["pageNum"], serde_json::json!(3));
}
