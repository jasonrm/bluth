# bluth

Declarative HTML rendering for Rust.

## Installation

```sh
cargo add bluth
```

## Example

An (untested) todo app with three nested components and axum:

```rust
use axum::{routing::get, Router};
use bluth::{Element, Signal, Html, Body, Head, Document, PatchElements};

// Signals for reactive state
#[derive(Signal)]
pub enum Signals {
    NewTodo(String),
}

// Inner component: individual todo item
#[derive(Element)]
#[element("li")]
#[attr(class = "todo-item")]
struct TodoItem {
    #[element("span")]
    text: String,

    #[element("button")]
    #[attr(data_on_click = "@delete('/todos/{id}')")]
    delete_btn: &'static str,

    #[skip]
    id: u32,
}

// Middle component: the todo list
#[derive(Element)]
#[element("ul")]
#[attr(id = "todo-list", class = "todos")]
struct TodoList {
    #[element]
    items: Vec<TodoItem>,
}

// Outer component: full page
#[derive(Element)]
#[element("div")]
#[attr(class = "container")]
struct TodoPage {
    #[element("h1")]
    title: &'static str,

    #[element("form")]
    #[attr(data_on_submit__prevent = "@post('/todos')")]
    form: TodoForm,

    #[element]
    list: TodoList,
}

#[derive(Element)]
#[element]
struct TodoForm {
    #[element("input")]
    #[attr(type = "text", placeholder = "New todo...", data_bind = NewTodo)]
    input: (),

    #[element("button")]
    #[attr(type = "submit")]
    submit: &'static str,
}

// Axum handlers
async fn index() -> Document<TodoPage> {
    let items = vec![
        TodoItem { text: "Learn Rust".into(), delete_btn: "×", id: 1 },
        TodoItem { text: "Build with Bluth".into(), delete_btn: "×", id: 2 },
    ];

    Document(
        Html {
            head: Head {
                title: "Todo App",
                extra: r#"<script type="module" src="https://cdn.jsdelivr.net/npm/@sudodevnull/datastar"></script>"#,
            },
            body: Body {
                signals: Some(Signals::NewTodo(String::new())),
                content: TodoPage {
                    title: "My Todos",
                    form: TodoForm {
                        input: (),
                        submit: "Add",
                    },
                    list: TodoList { items },
                },
            },
        }
    )
}

async fn add_todo(Signal(NewTodo(text)): Signal<NewTodo>) -> PatchElements<TodoList> {
    // In real app: save to database
    let items = vec![
        TodoItem { text, delete_btn: "×", id: 3 },
    ];
    PatchElements {
        selector: "#todo-list".into(),
        merge_mode: Some("prepend".into()),
        content: TodoList { items },
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
- Axum extractors: `Signal<T>`, `Signals<(A, B)>`
- SSE responses: `PatchElements`, `PatchSignals`

## License

MIT
