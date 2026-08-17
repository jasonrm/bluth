use crate::assets::Asset;
use crate::layout::{HtmlResponse, Nav};
use bluth::Element;
use bluth::datastar::{OnInterval, PatchElements, PatchMode};
use bluth::define_url;
use std::fmt::Display;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

define_url!(ItemUrl, "/catalog/item", item_id: u32);

#[derive(Element)]
#[element("main")]
#[attr(class = "max-w-3xl mx-auto px-6 py-12")]
struct CatalogPage {
    #[element]
    nav: Nav,
    #[element]
    title: CatalogTitle,
    #[element]
    intro: CatalogIntro,
    #[element]
    interpolate: CatalogRow<Interpolated>,
    #[element]
    format: CatalogRow<Formatted>,
    #[element]
    option: CatalogRow<MaybeName>,
    #[element]
    list: CatalogRow<Names>,
    #[element]
    void_bool: CatalogRow<Flag>,
    #[element]
    variant: CatalogRow<Tone>,
    #[element]
    fragment: CatalogRow<Loose>,
    #[element]
    interval: CatalogRow<Ticker>,
    #[element]
    url: CatalogRow<UrlDemo>,
}

#[derive(Element)]
#[element("h1")]
#[attr(class = "text-4xl font-bold text-white mb-2")]
struct CatalogTitle {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("p")]
#[attr(class = "text-gray-400 mb-8")]
struct CatalogIntro {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("article")]
#[attr(class = "mb-8 p-6 bg-gray-900 rounded-lg")]
struct CatalogRow<T>
where
    T: Display,
{
    #[element("h2")]
    #[attr(class = "text-xl font-semibold text-white mb-3")]
    title: &'static str,
    #[element("div")]
    #[attr(class = "mb-3 text-gray-200")]
    live: T,
    #[element]
    source: Source,
}

#[derive(Element)]
#[element("pre")]
#[attr(class = "bg-gray-950 rounded p-3 overflow-x-auto")]
struct Source {
    #[element("code")]
    #[attr(class = "text-sm text-green-400")]
    text: &'static str,
}

#[derive(Element)]
#[element("div")]
#[attr(id = "user-{user_id}", "data-config" = "{{ok: true}}")]
struct Interpolated {
    user_id: u32,
}

#[derive(Element)]
#[element("span")]
#[format("{} @ {:.1}", point.x, point.y)]
struct Formatted {
    point: Point,
}

struct Point {
    x: f64,
    y: f64,
}

#[derive(Element)]
#[element("span")]
struct MaybeName {
    #[element]
    #[map_or("anonymous")]
    name: Option<&'static str>,
}

#[derive(Element)]
#[element("ul")]
#[attr(class = "list-disc pl-5")]
struct Names {
    #[element]
    items: Vec<NameItem>,
}

#[derive(Element)]
#[element("li")]
struct NameItem {
    #[element]
    text: &'static str,
}

#[derive(Element)]
#[element("label")]
#[attr(class = "flex gap-2 items-center")]
struct Flag {
    #[element]
    input: FlagInput,
    #[element]
    label: &'static str,
}

#[derive(Element)]
#[element("input")]
#[attr("type" = "checkbox")]
struct FlagInput {
    #[attr]
    disabled: bool,
}

#[derive(Element)]
#[element("span")]
enum Tone {
    Ok(&'static str),
    Warn(&'static str),
}

#[derive(Element)]
struct Loose {
    #[element]
    prefix: &'static str,
    #[element]
    name: &'static str,
}

#[derive(Element)]
#[element("span")]
#[attr(id = "catalog-tick", "{interval}" = "@get('/catalog/tick')")]
struct Ticker {
    interval: OnInterval,
    #[element]
    label: TickLabel,
}

#[derive(Element)]
#[map_or("…")]
struct TickLabel(Option<String>);

#[derive(Element)]
struct TickDisplay {
    #[element]
    text: String,
}

#[derive(Element)]
#[element("p")]
#[format("{} → {} (id {})", pattern, path, id)]
struct UrlDemo {
    pattern: &'static str,
    path: String,
    id: u32,
}

impl<T: Display> CatalogRow<T> {
    fn new(title: &'static str, live: T, source: &'static str) -> Self {
        Self {
            title,
            live,
            source: Source { text: source },
        }
    }
}

impl CatalogPage {
    pub fn new() -> Self {
        let url = ItemUrl::new(7);
        Self {
            nav: Nav::site(),
            title: CatalogTitle { text: "Catalog" },
            intro: CatalogIntro {
                text: "Each row is a real Element value. The source is a string field, not a second markup language.",
            },
            interpolate: CatalogRow::new(
                "{field} and {{ }}",
                Interpolated { user_id: 42 },
                r##"#[attr(id = "user-{user_id}", "data-config" = "{{ok: true}}")]"##,
            ),
            format: CatalogRow::new(
                "#[format]",
                Formatted {
                    point: Point { x: 1.25, y: 2.5 },
                },
                r#"#[format("{} @ {:.1}", point.x, point.y)]"#,
            ),
            option: CatalogRow::new(
                "#[map_or]",
                MaybeName { name: None },
                r#"#[map_or("anonymous")] name: Option<&'static str>"#,
            ),
            list: CatalogRow::new(
                "Vec children",
                Names {
                    items: vec![
                        NameItem { text: "Element" },
                        NameItem { text: "Signal" },
                        NameItem { text: "OnInterval" },
                    ],
                },
                r#"#[element("li")] struct NameItem { text }  items: Vec<NameItem>"#,
            ),
            void_bool: CatalogRow::new(
                "() and bool attrs",
                Flag {
                    input: FlagInput { disabled: true },
                    label: "disabled",
                },
                r#"checkbox: (); #[attr] disabled: bool"#,
            ),
            variant: CatalogRow::new(
                "enum variants",
                Tone::Warn("careful"),
                r#"enum Tone { Ok(&'static str), Warn(&'static str) }"#,
            ),
            fragment: CatalogRow::new(
                "fragment (no wrapper)",
                Loose {
                    prefix: "hello ",
                    name: "world",
                },
                r#"struct Loose { prefix, name } // no #[element(\"tag\")] on the struct"#,
            ),
            interval: CatalogRow::new(
                "OnInterval",
                Ticker {
                    interval: OnInterval {
                        duration: Duration::from_secs(2),
                        leading: true,
                        view_transition: false,
                    },
                    label: TickLabel(None),
                },
                r#"OnInterval { duration, leading: true, view_transition: false }"#,
            ),
            url: CatalogRow::new(
                "define_url!",
                UrlDemo {
                    pattern: ItemUrl::PATTERN,
                    path: url.path(),
                    id: url.item_id,
                },
                r#"define_url!(ItemUrl, "/catalog/item", item_id: u32); url.path(); url.item_id"#,
            ),
        }
    }

    pub fn document(self, datastar: Asset, styles: Asset) -> bluth::Document<Self> {
        crate::layout::document(self, datastar, styles)
    }
}

pub struct Catalog;

impl Catalog {
    pub async fn get() -> impl axum::response::IntoResponse {
        HtmlResponse(CatalogPage::new().document(Asset::datastar(), Asset::styles()))
    }

    fn tick_patch() -> PatchElements<TickDisplay> {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        PatchElements {
            selector: Some("#catalog-tick".into()),
            mode: PatchMode::Inner,
            ..PatchElements::new(vec![TickDisplay {
                text: format!("tick {secs}"),
            }])
        }
    }

    pub async fn tick() -> impl axum::response::IntoResponse {
        Self::tick_patch()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_writes_id_and_unescapes_braces() {
        let html = Interpolated { user_id: 42 }.to_string();
        assert_eq!(html, r#"<div id="user-42" data-config="{ok: true}"></div>"#);
    }

    #[test]
    fn format_writes_point() {
        let html = Formatted {
            point: Point { x: 1.25, y: 2.5 },
        }
        .to_string();
        assert_eq!(html, "<span>1.25 @ 2.5</span>");
    }

    #[test]
    fn map_or_writes_fallback() {
        assert_eq!(
            MaybeName { name: None }.to_string(),
            "<span>anonymous</span>"
        );
        assert_eq!(
            MaybeName { name: Some("ada") }.to_string(),
            "<span>ada</span>"
        );
    }

    #[test]
    fn vec_writes_items() {
        let html = Names {
            items: vec![NameItem { text: "a" }, NameItem { text: "b" }],
        }
        .to_string();
        assert_eq!(
            html,
            r#"<ul class="list-disc pl-5"><li>a</li><li>b</li></ul>"#
        );
    }

    #[test]
    fn void_and_bool_attrs() {
        let html = Flag {
            input: FlagInput { disabled: true },
            label: "disabled",
        }
        .to_string();
        assert!(html.contains(r#"<input type="checkbox" disabled/>"#));
        assert!(html.contains("disabled</label>"));
    }

    #[test]
    fn enum_variant_and_fragment() {
        assert_eq!(Tone::Ok("good").to_string(), "<span>good</span>");
        assert_eq!(Tone::Warn("careful").to_string(), "<span>careful</span>");
        assert_eq!(
            Loose {
                prefix: "hello ",
                name: "world",
            }
            .to_string(),
            "hello world"
        );
    }

    #[test]
    fn interval_is_attribute_name() {
        let html = Ticker {
            interval: OnInterval {
                duration: Duration::from_secs(2),
                leading: true,
                view_transition: false,
            },
            label: TickLabel(None),
        }
        .to_string();
        assert!(html.contains("data-on-interval__duration.2s.leading="));
        assert!(html.contains(r#"id="catalog-tick""#));
    }

    #[test]
    fn define_url_fields_and_path() {
        let url = ItemUrl::new(7);
        assert_eq!(ItemUrl::PATTERN, "/catalog/item/{item_id}");
        assert_eq!(url.path(), "/catalog/item/7");
        assert_eq!(url.item_id, 7);
        let html = UrlDemo {
            pattern: ItemUrl::PATTERN,
            path: url.path(),
            id: url.item_id,
        }
        .to_string();
        assert_eq!(
            html,
            "<p>/catalog/item/{item_id} → /catalog/item/7 (id 7)</p>"
        );
    }

    #[test]
    fn tick_is_inner_patch() {
        let sse = Catalog::tick_patch().to_string();
        assert!(sse.contains("event: datastar-patch-elements"));
        assert!(sse.contains("data: selector #catalog-tick\n"));
        assert!(sse.contains("data: mode inner\n"));
        assert!(sse.contains("data: elements tick "));
    }
}
