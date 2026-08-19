# bluth

Typed HTML for Rust. Markup is ordinary structs that derive `Element`; the HTML is `Display` of those values. Not a template engine.

## Installation

```sh
cargo add bluth
```

## Example

An (untested) todo app with three nested components and axum:

```rust
use axum::{routing::get, Router};
use bluth::datastar::{PatchElements, PatchMode};
use bluth::{Body, Document, Element, Head, Html, Meta, Script, Signal};

#[derive(Signal)]
pub enum Signals {
    NewTodo(String),
}

#[derive(Element)]
#[element("li")]
#[attr(class = "todo-item")]
struct TodoItem {
    id: u32,

    #[element("span")]
    text: String,

    #[element("button")]
    #[attr("data-on:click" = "@delete('/todos/{id}')")]
    delete_btn: &'static str,
}

#[derive(Element)]
#[element("ul")]
#[attr(id = "todo-list", class = "todos")]
struct TodoList {
    #[element]
    items: Vec<TodoItem>,
}

#[derive(Element)]
#[element("div")]
#[attr(class = "container")]
struct TodoPage {
    #[element("h1")]
    title: &'static str,

    #[element("form")]
    #[attr("data-on:submit__prevent" = "@post('/todos')")]
    form: TodoForm,

    #[element]
    list: TodoList,
}

#[derive(Element)]
struct TodoForm {
    #[element("input")]
    #[attr("type" = "text", placeholder = "New todo...", data_bind = NewTodo)]
    input: (),

    #[element("button")]
    #[attr("type" = "submit")]
    submit: &'static str,
}

async fn index() -> Document<TodoPage> {
    let items = vec![
        TodoItem { id: 1, text: "Learn Rust".into(), delete_btn: "×" },
        TodoItem { id: 2, text: "Build with Bluth".into(), delete_btn: "×" },
    ];

    Document::new(Html {
        lang: "en",
        head: Head {
            meta: vec![Meta {
                charset: Some("utf-8"),
                name: None,
                content: None,
            }],
            title: "My Todos",
            link: vec![],
            script: vec![Script {
                src: "https://cdn.jsdelivr.net/gh/starfederation/datastar@v1.0.2/bundles/datastar.js",
                async_: false,
                type_: "module",
            }],
        },
        body: Body {
            class: "container",
            children: vec![TodoPage {
                title: "My Todos",
                form: TodoForm {
                    input: (),
                    submit: "Add",
                },
                list: TodoList { items },
            }],
        },
    })
}

async fn add_todo(Signal(text): Signal<NewTodo>) -> PatchElements<TodoList> {
    let items = vec![TodoItem { id: 3, text, delete_btn: "×" }];
    PatchElements {
        selector: Some("#todo-list".into()),
        mode: PatchMode::Prepend,
        ..PatchElements::new(vec![TodoList { items }])
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/todos", axum::routing::post(add_todo));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Features

- `#[derive(Element)]` — Convert structs to HTML
- `#[derive(Signal)]` — Type-safe reactive signals
- `#[attr(...)]` — HTML attributes with interpolation
- `#[element("tag")]` — Wrap fields in HTML tags
- Axum extractors: `Signal<T>`, `SignalMap`
- SSE responses: `PatchElements`, `PatchSignals`

## License

MIT
