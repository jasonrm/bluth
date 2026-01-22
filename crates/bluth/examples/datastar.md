# Bluth Framework Guide for LLMs

This guide explains how to create HTML elements using the Bluth framework with Datastar integration for reactivity.

## Core Structure

Every element uses the `#[derive(Element)]` macro with optional configuration attributes:

```rust
#[derive(Element)]
#[element("div")]
#[attr(id = "my-element", class = "container")]
struct MyComponent {
    // fields become content or children
}
```

## Element Definition

### Basic Element

- Use `#[element("tag")]` to specify the HTML tag (e.g., `div`, `span`, `input`, `time`)
- Without `#[element("tag")]`, the struct renders only its children (no wrapper tag)
- Use `#[attr(...)]` for HTML attributes

```rust
#[derive(Element)]
#[element("time")]
#[attr(
    id = "date-time",
    datetime = "{date_time}",
    class = "rounded bg-gray-600 text-gray-300 px-2 py-2 tabular-nums"
)]
#[format("{} @ {}", date_time.format("%Y-%m-%d"), date_time.format("%H:%M:%S"))]
pub struct DateTimeDisplay {
    date_time: DateTime<Utc>,
}
```

### Attribute Interpolation

Use `{field_name}` to interpolate field values into attributes:

```rust
#[derive(Element)]
#[element("div")]
#[attr(id = "user-{user_id}", data_name = "{name}")]
struct UserCard {
    user_id: u64,
    name: String,
}
```

## Nested Elements (Children)

Mark fields as child elements with `#[element]` or `#[element("tag")]`:

```rust
#[derive(Element)]
#[element("div")]
struct Container {
    #[element("h1")]
    #[attr(class = "title")]
    title: String,

    #[element("p")]
    description: String,

    #[element]  // No tag wrapper, just renders the child
    footer: FooterComponent,
}
```

### Empty Elements

Use `()` as the field type for self-closing or empty elements:

```rust
#[derive(Element)]
struct Form {
    #[element("input")]
    #[attr(type = "text", name = "username")]
    username_input: (),

    #[element("input")]
    #[attr(type = "submit", value = "Submit")]
    submit_button: (),
}
```

### Collections

Use `Vec<T>` for repeating elements:

```rust
#[derive(Element)]
#[element("ul")]
struct TodoList {
    #[element("li")]
    items: Vec<String>,
}
```

## Formatting

### Custom Format Strings

Use `#[format("...")]` to customize element content:

```rust
#[derive(Element)]
#[element("span")]
#[format("({}, {})", point.x, point.y)]
struct PointDisplay {
    point: Point,
}
```

### Optional Values with Fallback

Use `#[map_or("default")]` for `Option` types:

```rust
#[derive(Element)]
#[element("td")]
#[map_or("-")]
struct PriceCell {
    price: Option<u64>,
}
```

Combined with format:

```rust
#[element("td")]
#[map_or("-")]
#[format("{:.2}")]
value: Option<f64>,
```

## Signals (Reactive State)

### Defining Signals

Create a signal enum with `#[derive(Signal)]`:

```rust
#[derive(Signal)]
pub enum AppSignals {
    UserName(String),
    SearchTerm(Option<String>),
    #[signal(name = "pageNum")]  // Custom signal name
    PageNumber(i32),
}
```

### Using Signals in Elements

Use `SignalValue<T>` to hold signal values:

```rust
#[derive(Element)]
struct SearchBar {
    search_term: SignalValue<SearchTerm>,

    #[element("input")]
    #[attr(data_bind = search_term, id = "search-input")]
    input: (),
}
```

### Signal Binding Methods

1. **Selector type**: `data_bind = UserName` (uses the signal type directly)
2. **Field reference**: `data_bind = search_term` (field has `SignalValue<SearchTerm>` type)
3. **String literal**: `data_bind = "legacySignal"` (raw string)

## Datastar Attributes

Use quoted attribute names for datastar attributes with special characters.

### Core Attributes

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `data_bind` | Two-way binding to signal | `data_bind = search_term` |
| `data-signals:name` | Define local signal | `"data-signals:_{id}" = "false"` |
| `data-text` | Reactive text content | `data_text = "$userName"` |
| `data-show` | Conditional visibility | `"data-show" = "$isVisible"` |
| `data-class:classname` | Conditional class | `"data-class:hidden" = "!$isOpen"` |
| `data-attr:attrname` | Dynamic attribute | `"data-attr:disabled" = "$isLoading"` |
| `data-style` | Dynamic styles | `"data-style" = "{ color: $textColor }"` |
| `data-computed` | Computed expression | `"data-computed" = r#"fullName = $first + " " + $last"#` |
| `data-ref` | Element reference | `data_ref = "myElement"` |
| `data-init` | Initialization expression | `"data-init" = "$count = 0"` |
| `data-effect` | Side effects | `"data-effect" = "console.log($value)"` |
| `data-indicator` | Loading indicator | `data_indicator = "loading"` |
| `data-ignore` | Skip processing | `data_ignore` |
| `data-ignore-morph` | Skip morphing | `data_ignore_morph` |
| `data-preserve-attr` | Preserve during morph | `data_preserve_attr = "value"` |
| `data-json-signals` | JSON signal init | `"data-json-signals" = r#"{"items": []}"#` |

### Event Handlers

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `data-on:event` | Event handler | `"data-on:click" = "$count++"` |
| `data-on:event__modifier` | With modifier | `"data-on:input__debounce.200ms" = "@post('/search')"` |
| `data-on:click__outside` | Click outside | `"data-on:click__outside" = "$isOpen = false"` |
| `data-on-intersect` | Intersection observer | `"data-on-intersect" = "@get('/load-more')"` |
| `data-on-interval` | Interval-based | `"data-on-interval.1000ms" = "@get('/refresh')"` |
| `data-on-signal-patch` | Signal change | `"data-on-signal-patch" = "console.log('changed')"` |
| `data-on-signal-patch-filter` | Filtered signal change | `"data-on-signal-patch-filter" = "userName"` |

### Event Modifiers

Common modifiers for `data-on`:
- `__debounce.Xms` - Debounce the event
- `__throttle.Xms` - Throttle the event
- `__once` - Only fire once
- `__prevent` - preventDefault()
- `__stop` - stopPropagation()
- `__outside` - Trigger on clicks outside element
- `__window` - Listen on window
- `__document` - Listen on document

### Server Actions

Use `@` prefix for server actions:
- `@get('/path')` - GET request
- `@post('/path')` - POST request
- `@put('/path')` - PUT request
- `@patch('/path')` - PATCH request
- `@delete('/path')` - DELETE request

## Dynamic Classes (CVA Pattern)

For complex class logic, compute classes in a method:

```rust
#[derive(Element)]
#[element("button")]
#[attr(class = self.class())]
pub struct Button<T: Display> {
    intent: ButtonIntent,
    size: ButtonSize,
    disabled: bool,

    #[element]
    content: T,
}

impl<T: Display> Button<T> {
    fn class(&self) -> String {
        let mut classes = vec!["btn"];
        
        match self.intent {
            ButtonIntent::Primary => classes.push("btn-primary"),
            ButtonIntent::Secondary => classes.push("btn-secondary"),
        }
        
        if self.disabled {
            classes.push("opacity-50 cursor-not-allowed");
        }
        
        classes.join(" ")
    }
}
```

## Unique IDs for Dynamic Components

Use `Ulid` for generating unique IDs:

```rust
use ulid::Ulid;

#[derive(Element)]
#[element("div")]
#[attr(
    id = "_{id}",
    "data-signals:_{id}" = "false",
    "data-on:click__outside" = "$_{id} = false",
)]
pub struct Dropdown<T: Display> {
    id: Ulid,

    #[element("button")]
    #[attr("data-on:click" = "$_{id} = !$_{id}")]
    toggle: Button<T>,

    #[element("div")]
    #[attr("data-class:hidden" = "!$_{id}")]
    content: T,
}

impl<T: Display> Dropdown<T> {
    pub fn new(toggle: Button<T>, content: T) -> Self {
        Self {
            id: Ulid::new(),
            toggle,
            content,
        }
    }
}
```

## SSE/Patch Responses

Return elements as patch responses for live updates:

```rust
pub async fn update_results(
    SignalExtractor(search_term): SignalExtractor<SearchTerm>,
) -> Result<impl IntoResponse, AppError> {
    let content = SearchResults {
        results: search_term,
    };

    Ok(PatchElements::new(vec![content]))
}
```

## Complete Example

```rust
use bluth::{Element, Signal, SignalValue};
use ulid::Ulid;

#[derive(Signal)]
pub enum Signals {
    SearchQuery(Option<String>),
    IsLoading(bool),
}

#[derive(Element)]
#[element("div")]
#[attr(id = "search-results")]
#[map_or("No results found")]
pub struct SearchResults {
    results: Option<String>,
}

#[derive(Element)]
#[element("div")]
#[attr(class = "search-container")]
pub struct SearchBar {
    search_query: SignalValue<SearchQuery>,

    #[element("input")]
    #[attr(
        id = "search-input",
        data_bind = search_query,
        "data-on:input__debounce.300ms" = "@post('/api/search')",
        type = "text",
        placeholder = "Search...",
        class = "input input-bordered"
    )]
    input: (),

    #[element("div")]
    #[attr(
        "data-show" = "$isLoading",
        class = "loading-spinner"
    )]
    loading: (),

    #[element]
    results: SearchResults,
}

impl SearchBar {
    pub fn new(query: SignalValue<SearchQuery>) -> Self {
        Self {
            search_query: query.clone(),
            input: (),
            loading: (),
            results: SearchResults {
                results: query.into_inner(),
            },
        }
    }
}
```

## Best Practices

1. **Use unique IDs** - Generate with `Ulid::new()` for dynamic components
2. **Interpolate IDs in signals** - `"data-signals:_{id}"` ensures unique signal names
3. **Prefer signal types over strings** - `data_bind = SearchTerm` instead of `data_bind = "searchTerm"`
4. **Use raw strings for JS** - `r#"..."#` when embedding JavaScript with quotes
5. **Self-closing elements** - Use `()` as the field type for empty elements
6. **Composable components** - Nest elements by including them as fields with `#[element]`
7. **Avoid comments** - Code should be self-explanatory; only add comments for warnings or non-obvious behavior
