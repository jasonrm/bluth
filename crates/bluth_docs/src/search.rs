use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use bluth::datastar::{PatchElements, PatchMode, PatchSignals};
use bluth::{Element, Signal, SignalValue};
use std::fmt::Display;

const ITEMS: &[&str] = &[
    "Element",
    "Signal",
    "SignalValue",
    "PatchElements",
    "PatchSignals",
    "OnInterval",
    "define_url",
    "format",
    "map_or",
];

#[derive(Signal)]
pub enum SearchSignals {
    SearchQuery(String),
    IsLoading(bool),
}

#[derive(Element)]
#[element("section")]
#[attr(class = "mb-8 p-6 bg-gray-900 rounded-lg")]
pub struct SearchSection {
    #[element]
    heading: SearchHeading,
    #[element]
    hint: SearchHint,
    #[element]
    form: SearchForm,
    #[element("div")]
    #[attr(id = "search-results", class = "mt-4 text-gray-200")]
    results: SearchResults,
}

#[derive(Element)]
#[element("h2")]
#[attr(class = "text-xl font-semibold text-white mb-3")]
struct SearchHeading {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("p")]
#[attr(class = "text-gray-400 mb-3")]
struct SearchHint {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("form")]
#[attr(
    class = "flex gap-2",
    "data-on:submit__prevent" = "@post('/search')"
)]
struct SearchForm {
    query: SignalValue<SearchQuery>,

    #[element("input")]
    #[attr(
        "type" = "text",
        placeholder = "Search the API…",
        class = "flex-1 rounded bg-gray-800 px-3 py-2 text-white",
        data_bind = query,
        "data-on:input__debounce.200ms" = "@post('/search')",
        "data-indicator" = "$isLoading"
    )]
    input: (),

    #[element("button")]
    #[attr("type" = "submit", class = "rounded bg-green-700 px-3 py-2 text-white")]
    submit: &'static str,
}

#[derive(Element)]
#[map_or("No matches")]
struct SearchResults(Option<SearchList>);

#[derive(Element)]
#[element("ul")]
#[attr(class = "list-disc pl-5")]
struct SearchList {
    #[element]
    hits: Vec<SearchHit>,
}

#[derive(Element)]
#[element("li")]
struct SearchHit {
    #[element]
    text: &'static str,
}

pub struct SearchReply {
    signals: PatchSignals<SearchSignals>,
    elements: PatchElements<SearchResults>,
}

impl Display for SearchReply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.signals, self.elements)
    }
}

impl IntoResponse for SearchReply {
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/event-stream")],
            self.to_string(),
        )
            .into_response()
    }
}

pub struct Search;

impl SearchSection {
    pub fn new() -> Self {
        Self {
            heading: SearchHeading { text: "Search" },
            hint: SearchHint {
                text: "Bind a Signal, POST it, patch the results. That is the Datastar loop.",
            },
            form: SearchForm {
                query: SignalValue::new(String::new()),
                input: (),
                submit: "Search",
            },
            results: Search::results(""),
        }
    }
}

impl Search {
    fn results(query: &str) -> SearchResults {
        let needle = query.trim().to_lowercase();
        let hits: Vec<SearchHit> = ITEMS
            .iter()
            .filter(|item| needle.is_empty() || item.to_lowercase().contains(&needle))
            .map(|text| SearchHit { text })
            .collect();
        if hits.is_empty() {
            SearchResults(None)
        } else {
            SearchResults(Some(SearchList { hits }))
        }
    }

    pub fn reply(query: &str) -> SearchReply {
        SearchReply {
            signals: PatchSignals::new(vec![SearchSignals::IsLoading(false)]),
            elements: PatchElements {
                selector: Some("#search-results".into()),
                mode: PatchMode::Inner,
                ..PatchElements::new(vec![Self::results(query)])
            },
        }
    }

    pub async fn post(Signal(text): Signal<SearchQuery>) -> SearchReply {
        Self::reply(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::{FromRequest, Request};
    use axum::http::{Method, header};

    #[test]
    fn empty_query_lists_the_library() {
        let html = Search::results("").to_string();
        assert!(html.contains("<ul"));
        assert!(html.contains("PatchElements"));
        assert!(html.contains("OnInterval"));
    }

    #[test]
    fn unknown_query_is_map_or_fallback() {
        let html = Search::results("zzzz").to_string();
        assert_eq!(html, "No matches");
    }

    #[test]
    fn reply_is_inner_patch_and_signals() {
        let sse = Search::reply("patch").to_string();
        assert!(sse.contains("event: datastar-patch-signals"));
        assert!(sse.contains("event: datastar-patch-elements"));
        assert!(sse.contains("data: selector #search-results\n"));
        assert!(sse.contains("data: mode inner\n"));
        assert!(sse.contains("data: elements <ul"));
        assert!(sse.contains("PatchElements"));
        assert!(!sse.contains("OnInterval"));
    }

    #[tokio::test]
    async fn post_reads_datastar_signal() {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/search")
            .header(header::CONTENT_TYPE, "application/json")
            .header("Datastar-Request", "true")
            .body(Body::from(r#"{"searchQuery":"signal"}"#))
            .expect("request");

        let Signal(text) = Signal::<SearchQuery>::from_request(request, &())
            .await
            .expect("signal");
        let sse = Search::reply(&text).to_string();
        assert!(sse.contains("data: selector #search-results\n"));
        assert!(sse.contains("data: mode inner\n"));
        assert!(sse.contains("Signal"));
        assert!(sse.contains("SignalValue"));
        assert!(!sse.contains("PatchElements"));
    }
}
