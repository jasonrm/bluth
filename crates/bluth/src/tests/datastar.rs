use crate::{Element, Signal, SignalEnum, SignalMap, SignalName, SignalValue};

#[test]
fn data_bind_with_signal_selector() {
    #[derive(Signal)]
    enum BindSignals {
        UserName(String),
    }

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
    #[derive(Signal)]
    enum SearchSignals {
        SearchTerm(Option<String>),
    }

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
    #[derive(Signal)]
    enum PageSignals {
        #[signal(name = "pageNum")]
        PageNumber(i32),
    }

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
    #[derive(Signal)]
    enum NestedSignals {
        UserName(String),
        SearchTerm(Option<String>),
    }

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
    #[derive(Signal)]
    enum NameSignals {
        UserName(String),
        SearchTerm(Option<String>),
        #[signal(name = "pageNum")]
        PageNumber(i32),
    }

    assert_eq!(UserName::NAME, "userName");
    assert_eq!(SearchTerm::NAME, "searchTerm");
    assert_eq!(PageNumber::NAME, "pageNum");
}

#[test]
fn selector_as_ref_str() {
    #[derive(Signal)]
    enum RefSignals {
        UserName(String),
        SearchTerm(Option<String>),
        #[signal(name = "pageNum")]
        PageNumber(i32),
    }

    assert_eq!(UserName.as_ref(), "userName");
    assert_eq!(SearchTerm.as_ref(), "searchTerm");
    assert_eq!(PageNumber.as_ref(), "pageNum");
}

#[test]
fn wrap_and_extract() {
    #[derive(Signal)]
    enum WrapSignals {
        UserName(String),
        #[signal(name = "pageNum")]
        PageNumber(i32),
    }

    let signal = UserName::from_value("hello".to_string());
    assert!(matches!(&signal, WrapSignals::UserName(s) if s == "hello"));

    let extracted = UserName::value(&signal);
    assert_eq!(extracted, Some(&"hello".to_string()));

    let wrong_signal = PageNumber::from_value(42);
    assert_eq!(UserName::value(&wrong_signal), None);
}

#[test]
fn into_inner() {
    #[derive(Signal)]
    enum OwnedSignals {
        UserName(String),
        SearchTerm(Option<String>),
    }

    let signal = SearchTerm::from_value(Some("query".to_string()));
    let inner = SearchTerm::owned(signal);
    assert_eq!(inner, Some(Some("query".to_string())));

    let wrong_signal = UserName::from_value("test".to_string());
    let inner = SearchTerm::owned(wrong_signal);
    assert_eq!(inner, None);
}

#[test]
fn signal_enum_signal_name() {
    #[derive(Signal)]
    enum NameSignals {
        UserName(String),
        SearchTerm(Option<String>),
        #[signal(name = "pageNum")]
        PageNumber(i32),
    }

    let signal = NameSignals::UserName("test".to_string());
    assert_eq!(signal.name(), "userName");

    let signal = NameSignals::SearchTerm(None);
    assert_eq!(signal.name(), "searchTerm");

    let signal = NameSignals::PageNumber(1);
    assert_eq!(signal.name(), "pageNum");
}

#[test]
fn signal_enum_to_json_value() {
    #[derive(Signal)]
    enum JsonSignals {
        UserName(String),
        SearchTerm(Option<String>),
        #[signal(name = "pageNum")]
        PageNumber(i32),
    }

    let signal = JsonSignals::UserName("test".to_string());
    assert_eq!(signal.json().expect("json"), serde_json::json!("test"));

    let signal = JsonSignals::SearchTerm(Some("query".to_string()));
    assert_eq!(signal.json().expect("json"), serde_json::json!("query"));

    let signal = JsonSignals::SearchTerm(None);
    assert_eq!(signal.json().expect("json"), serde_json::Value::Null);

    let signal = JsonSignals::PageNumber(42);
    assert_eq!(signal.json().expect("json"), serde_json::json!(42));
}

#[test]
fn signal_enum_serialize() {
    #[derive(Signal)]
    enum SerializeSignals {
        UserName(String),
        #[signal(name = "pageNum")]
        PageNumber(i32),
    }

    let signal = SerializeSignals::UserName("john".to_string());
    let json = serde_json::to_string(&signal).unwrap();
    assert_eq!(json, r#"{"userName":"john"}"#);

    let signal = SerializeSignals::PageNumber(5);
    let json = serde_json::to_string(&signal).unwrap();
    assert_eq!(json, r#"{"pageNum":5}"#);
}

#[test]
fn signal_enum_clone() {
    #[derive(Signal)]
    enum CloneSignals {
        UserName(String),
    }

    let signal = CloneSignals::UserName("test".to_string());
    let cloned = signal.clone();
    assert!(matches!(cloned, CloneSignals::UserName(s) if s == "test"));
}

#[test]
fn signal_enum_debug() {
    #[derive(Signal)]
    enum DebugSignals {
        UserName(String),
    }

    let signal = DebugSignals::UserName("test".to_string());
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
    #[derive(Signal)]
    enum MergeSignals {
        UserName(String),
        #[signal(name = "pageNum")]
        PageNumber(i32),
    }

    let signals = vec![
        MergeSignals::UserName("john".to_string()),
        MergeSignals::PageNumber(3),
    ];

    let map = SignalMap::merge(&signals).expect("merge");

    assert_eq!(map.values["userName"], serde_json::json!("john"));
    assert_eq!(map.values["pageNum"], serde_json::json!(3));
}
