use crate::assets::Asset;
use crate::layout::HtmlResponse;
use bluth::{Body, Document, Element, Head, Html, Link, Script};

#[derive(Element)]
#[element("main")]
#[attr(class = "max-w-3xl mx-auto px-6 py-12")]
struct HomePage {
    #[element]
    header: Header,
    #[element]
    ticker: TickerSection,
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
            header: Header {
                title: Title { text: "bluth" },
                subtitle: Subtitle {
                    text: "Type-safe HTML components for Rust",
                },
            },
            ticker: TickerSection {
                heading: TickerHeading { text: "SSE Demo" },
                ticker: TickerDisplay { text: "" },
            },
            intro: Intro {
                text: IntroText {
                    text: "bluth is a Rust library for building HTML with compile-time safe, composable components. Define your markup as structs, derive Element, and get type-checked HTML rendering with zero runtime overhead.",
                },
            },
            example: Example {
                heading: ExampleHeading {
                    text: "Quick Start",
                },
                code: CodeBlock {
                    code: Code {
                        text: r#"use bluth::Element;

#[derive(Element)]
#[element("div")]
#[attr(class = "greeting")]
struct Hello {
    #[element("span")]
    who: String,
}

let hello = Hello { who: "world".into() };
// renders: <div class="greeting"><span>world</span></div>"#,
                    },
                },
            },
        }
    }

    pub fn document(self, datastar: Asset, styles: Asset) -> Document<Self> {
        Document::new(Html {
            lang: "en",
            head: Head {
                link: vec![Link {
                    id: Some("stylesheet"),
                    href: styles.url,
                }],
                script: vec![Script {
                    src: datastar.url,
                    async_: false,
                    type_: "module",
                }],
            },
            body: Body {
                class: "bg-gray-950 text-gray-100 min-h-screen",
                children: vec![self],
            },
        })
    }
}

impl Home {
    pub async fn get() -> impl axum::response::IntoResponse {
        HtmlResponse(HomePage::new().document(Asset::datastar(), Asset::styles()))
    }
}
