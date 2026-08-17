# AGENTS.md

Bluth is typed HTML, not a template engine. Markup is ordinary Rust structs that derive `Element`: the type is the component, fields are children or text, and `#[element]` / `#[attr]` / `#[format]` / `#[map_or]` are compile-time annotations on that type. `{field}` in an attribute names a field of the same struct. There are no template files and no runtime fill-in of a string against a context map — the HTML is `Display` of values.

That is the opposite of Tera, Handlebars, Askama, and similar crates, where a separate document is written in another syntax and later interpolated. Do not add those. Do not call this crate a template engine.

`#[derive(Element)]` and `#[derive(Signal)]` are the product. They exist to turn a struct or enum into `Display` / a signal. Public fields are the composition API: a page is a struct literal, not a builder and not a bag of setters. Adding getters to hide those fields is worse than leaving them public.

Keep new APIs in that shape: compose elements as values, return typed Datastar/SSE patches (`PatchElements`, `PatchSignals`) and typed signals.

## Construction

Write the value. Do not fluent-chain methods that take `mut self` (or `&self`) and return `Self` to nibble on optional fields. That includes `with_*` helpers. Struct update (`Type { field: x, ..base }`) is the way to override defaults.

`new` only assigns fields. It may fill Datastar/HTML defaults that the wire format would omit anyway. It does no I/O, parse, or other work. Prefer a struct literal when every field is meaningful and there are no defaults.

Patches and intervals keep the same wire behavior they have today. They stop being builders.

```rust
// Default outer morph of these elements (today: PatchElements::new(vec![el])).
PatchElements::new(vec![el])

// Addressed inner patch (today: PatchElements::new(vec![el]).selector("#ticker-text").mode(PatchMode::Inner)).
PatchElements {
    selector: Some("#ticker-text".into()),
    mode: PatchMode::Inner,
    ..PatchElements::new(vec![el])
}

// Signals (today: PatchSignals::new(vec![s]).only_if_missing(true)).
PatchSignals {
    only_if_missing: true,
    ..PatchSignals::new(vec![s])
}

// Interval attribute (today: DatastarInterval::new(d).leading().viewtransition()).
OnInterval {
    duration: Duration::from_secs(1),
    leading: true,
    view_transition: true,
}
```

Store Datastar defaults as values, not `Option` flags that mean "use the default": `mode` is `PatchMode` (default `Outer`), `use_view_transition` / `only_if_missing` / `leading` are `bool` (default `false`). `selector` and `namespace` stay `Option` because absence is real (no selector, HTML rather than SVG/MathML). `Display` still omits default/absent fields from the SSE.

`Document::new(html)` may keep filling `<!doctype html>`. A struct literal can still set a different doctype. Library HTML types (`Html`, `Body`, `Head`, `Meta`, `Link`, `Script`) are struct literals.

Axum extractors stay destructurable tuple structs (`Signal(NewTodo(text))`). That public field is the point.

`define_url!` stays a value type with public fields, a pattern, and `path()`. Do not turn it into a builder.

## Rust

Name types after entities, not roles or `-er` (`FileContent` not `FileReader`). Name each impl `{Qualifier}{FullTrait}` (`DatabasePostRepository` for `PostRepository`), never abbreviating the trait. Name the protocol concept (`PatchElements`, `OnInterval`), not a vendor wrapper (`DatastarClient`). Queries are nouns (`title()`), effects are verbs returning `()` or `Result<()>` (`save()`). Locals are one word matching the type, else extract.

A concrete struct is a thing in the domain (an element, a patch, a signal, a request), not an orchestrator. I/O lives on a skinny resource that holds a handle; parse and deserialize outside it. Prefer concrete types over extra generics. Wrap a collection when the collection has behavior (`Library` not `Vec<Book>`); a `Vec` of child elements is fine. The macros crate is a compiler plugin: attribute specs and emit helpers are allowed there.

Never mutate via `&mut self`. Fields are written at construction. `DerefMut` on a wrapper is mutation — construct a new value instead. `Cell` / `RefCell` only for caches or foreign mutable resources. Return lazy objects (`Display`, `Iterator`, `Future`) and run work at a terminal (`to_string`, `collect`, `await`).

No `get_` / `set_`. Public fields replace both. Pass whole objects. `Option` is allowed where absence is real (missing selector, optional attribute, missing signal in a request). Convert `Option` at the HTTP / serde boundary when the domain value is required. Every public method except `fn new(...) -> Self` takes `&self` or `self`. Domain literals live on the type that uses them, not a loose `pub const` module.

Compose in `main` or a factory. No DI containers, setter injection, or `static` / `lazy_static` / `once_cell` globals. Inject collaborators as fields. Do not add a static logger.

Do not add Tera-style context bags, a second markup language in a file, or runtime interpolators. `#[format]`, `#[map_or]`, and `{field}` interpolation stay as derive annotations. Extra behavior belongs on a type (`Display` of a wrapper), not a new guest syntax.

No ORM / ActiveRecord / DAO. Ban Singleton, Facade, Mediator, Visitor. No mixin traits. Never `Any` / `TypeId` downcast or branch on a trait object's concrete type.

Fail immediately on broken invariants. One error type per crate. Never catch only to log; `?` until the outermost conversion. Attach `map_err` / `with_context` per origin; keep `?` scopes small. Do not put statements after `return` / `panic!` / `continue` / `break` inside `else` — un-indent them. One exit per method.

Ship fakes (trait impls) in the production crate when the crate owns the trait. Each test owns its literals and objects; no shared fixtures or helper builders. No logging in unit tests. Exercise the public API (`Display`, extractors, struct fields, signal traits). Do not reach into private items or use unsafe / reflection.
