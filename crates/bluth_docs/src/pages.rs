use crate::assets::Asset;
use crate::layout::{HtmlResponse, Nav};
use crate::search::SearchSection;
use bluth::Element;

#[derive(Element)]
#[element("main")]
#[attr(class = "max-w-3xl mx-auto px-6 py-12")]
struct HomePage {
    #[element]
    nav: Nav,
    #[element]
    header: Header,
    #[element]
    ticker: TickerSection,
    #[element]
    search: SearchSection,
    #[element]
    intro: Intro,
    #[element]
    example: Example,
}

#[derive(Element)]
#[element("header")]
#[attr(class = "mb-8")]
struct Header {
    #[element]
    title: Title,
    #[element]
    subtitle: Subtitle,
}

#[derive(Element)]
#[element("h1")]
#[attr(class = "text-4xl font-bold text-white")]
struct Title {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("p")]
#[attr(class = "mt-2 text-lg text-gray-400")]
struct Subtitle {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("section")]
#[attr(class = "mb-8 p-6 bg-gray-900 rounded-lg")]
struct TickerSection {
    #[element]
    heading: TickerHeading,
    #[element]
    ticker: TickerDisplay,
}

#[derive(Element)]
#[element("h2")]
#[attr(class = "text-xl font-semibold text-white mb-3")]
struct TickerHeading {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("div")]
#[attr(class = "text-2xl font-mono text-green-400")]
#[attr("data-init" = "@get('/ticker')")]
struct TickerDisplay {
    #[element("span")]
    #[attr(id = "ticker-text")]
    text: &'static str,
}

#[derive(Element)]
#[element("section")]
#[attr(class = "mb-8")]
struct Intro {
    #[element]
    text: IntroText,
}

#[derive(Element)]
#[element("p")]
#[attr(class = "text-gray-300 leading-relaxed")]
struct IntroText {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("section")]
#[attr(class = "mb-8")]
struct Example {
    #[element]
    heading: ExampleHeading,
    #[element]
    code: CodeBlock,
}

#[derive(Element)]
#[element("h2")]
#[attr(class = "text-2xl font-semibold text-white mb-4")]
struct ExampleHeading {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("pre")]
#[attr(class = "bg-gray-900 rounded-lg p-4 overflow-x-auto")]
struct CodeBlock {
    #[element]
    code: Code,
}

#[derive(Element)]
#[element("code")]
#[attr(class = "text-sm text-green-400")]
struct Code {
    #[element]
    text: &'static str,
}

pub struct Home;

impl HomePage {
    pub fn new() -> Self {
        Self {
            nav: Nav::site(),
            header: Header {
                title: Title { text: "bluth" },
                subtitle: Subtitle {
                    text: "Typed HTML for Rust — structs, not templates",
                },
            },
            ticker: TickerSection {
                heading: TickerHeading { text: "SSE Demo" },
                ticker: TickerDisplay { text: "" },
            },
            search: SearchSection::new(),
            intro: Intro {
                text: IntroText {
                    text: "Bluth is typed HTML, not a template engine. Markup is ordinary Rust structs that derive Element: the type is the component, fields are children or text, and the HTML is Display of those values. A page is a struct literal. Public fields are the composition API.",
                },
            },
            example: Example {
                heading: ExampleHeading {
                    text: "Quick Start",
                },
                code: CodeBlock {
                    code: Code {
                        text: r##"use bluth::datastar::{PatchElements, PatchMode};
use bluth::Element;

#[derive(Element)]
#[element("span")]
#[attr(id = "ticker-text")]
struct TickerText {
    #[element]
    text: String,
}

let patch = PatchElements {
    selector: Some("#ticker-text".into()),
    mode: PatchMode::Inner,
    ..PatchElements::new(vec![TickerText {
        text: "Hello World from Bluth".into(),
    }])
};"##,
                    },
                },
            },
        }
    }

    pub fn document(self, datastar: Asset, styles: Asset) -> bluth::Document<Self> {
        crate::layout::document(self, datastar, styles)
    }
}

impl Home {
    pub async fn get() -> impl axum::response::IntoResponse {
        HtmlResponse(HomePage::new().document(Asset::datastar(), Asset::styles()))
    }
}
