use proc_macro2::{Span, TokenStream};
use syn::parse::Parser;
use syn::{Attribute, Ident, Meta, Type};

#[derive(Debug, Clone)]
pub struct AttrSpec {
    pub key: AttrKey,
    pub value: AttrValue,
}

#[derive(Debug, Clone)]
pub enum AttrKey {
    Literal(String),
    Interpolated(String),
}

#[derive(Clone)]
pub enum AttrValue {
    Literal(String),
    Interpolated(String),
    Bool(bool),
    Path(syn::Path),
    SignalFieldBinding(syn::Ident),
    Expr(syn::Expr),
}

impl std::fmt::Debug for AttrValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttrValue::Literal(s) => f.debug_tuple("Literal").field(s).finish(),
            AttrValue::Interpolated(s) => f.debug_tuple("Interpolated").field(s).finish(),
            AttrValue::Bool(b) => f.debug_tuple("Bool").field(b).finish(),
            AttrValue::Path(_) => f.debug_tuple("Path").field(&"...").finish(),
            AttrValue::SignalFieldBinding(ident) => f
                .debug_tuple("SignalFieldBinding")
                .field(&ident.to_string())
                .finish(),
            AttrValue::Expr(_) => f.debug_tuple("Expr").field(&"...").finish(),
        }
    }
}

#[derive(Clone)]
pub struct FormatSpec {
    pub format_string: String,
    pub args: Option<TokenStream>,
}

impl std::fmt::Debug for FormatSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatSpec")
            .field("format_string", &self.format_string)
            .field("args", &self.args.as_ref().map(|_| "..."))
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct ElementSpec {
    pub tag: Option<String>,
    pub attrs: Vec<AttrSpec>,
    pub format: Option<FormatSpec>,
    pub map_or: Option<String>,
}

#[derive(Debug, Default)]
pub struct FieldSpec {
    pub tag: Option<String>,
    pub should_render: bool,
    pub attrs: Vec<AttrSpec>,
    pub format: Option<FormatSpec>,
    pub map_or: Option<String>,
    pub is_attr: bool,
    pub attr_rename: Option<String>,
}

impl ElementSpec {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut spec = ElementSpec::default();

        for attr in attrs {
            let path = attr.path();

            if path.is_ident("element") {
                spec.tag = Some(parse_single_string_arg(attr)?);
            } else if path.is_ident("format") {
                spec.format = Some(parse_format_args(attr)?);
            } else if path.is_ident("map_or") {
                spec.map_or = Some(parse_single_string_arg(attr)?);
            } else if path.is_ident("attr") {
                let parsed = parse_attr_attribute(attr)?;
                spec.attrs.extend(parsed);
            }
        }

        if !spec.attrs.is_empty() && spec.tag.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                "#[attr(...)] requires #[element(\"tag\")] to be specified",
            ));
        }

        Ok(spec)
    }
}

impl FieldSpec {
    pub fn from_attrs(
        attrs: &[Attribute],
        field_name: &Ident,
        field_type: &Type,
    ) -> syn::Result<Self> {
        let mut spec = FieldSpec::default();

        for attr in attrs {
            let path = attr.path();

            if path.is_ident("element") {
                spec.should_render = true;
                spec.tag = parse_optional_string_arg(attr)?;
            } else if path.is_ident("format") {
                spec.format = Some(parse_format_args(attr)?);
            } else if path.is_ident("map_or") {
                spec.map_or = Some(parse_single_string_arg(attr)?);
            } else if path.is_ident("attr") {
                let parsed = parse_field_attr_attribute(attr, field_name, field_type)?;
                match parsed {
                    FieldAttrResult::IsAttr { rename } => {
                        spec.is_attr = true;
                        spec.attr_rename = rename;
                    }
                    FieldAttrResult::Attrs(attrs) => {
                        spec.attrs.extend(attrs);
                    }
                }
            }
        }

        Ok(spec)
    }
}

fn parse_single_string_arg(attr: &Attribute) -> syn::Result<String> {
    let meta_list = attr.meta.require_list()?;
    let lit: syn::LitStr = syn::parse2(meta_list.tokens.clone())?;
    Ok(lit.value())
}

fn parse_format_args(attr: &Attribute) -> syn::Result<FormatSpec> {
    let meta_list = attr.meta.require_list()?;
    let tokens = meta_list.tokens.clone();

    let parser = |input: syn::parse::ParseStream| {
        let format_string: syn::LitStr = input.parse()?;

        let args = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            let remaining: TokenStream = input.parse()?;
            if remaining.is_empty() {
                None
            } else {
                Some(remaining)
            }
        } else {
            None
        };

        Ok(FormatSpec {
            format_string: format_string.value(),
            args,
        })
    };

    parser.parse2(tokens)
}

fn parse_optional_string_arg(attr: &Attribute) -> syn::Result<Option<String>> {
    match &attr.meta {
        Meta::Path(_) => Ok(None),
        Meta::List(list) => {
            if list.tokens.is_empty() {
                Ok(None)
            } else {
                let lit: syn::LitStr = syn::parse2(list.tokens.clone())?;
                Ok(Some(lit.value()))
            }
        }
        Meta::NameValue(_) => Err(syn::Error::new_spanned(
            attr,
            "unexpected name=value syntax",
        )),
    }
}

fn parse_attr_attribute(attr: &Attribute) -> syn::Result<Vec<AttrSpec>> {
    let mut attrs = Vec::new();
    let meta_list = attr.meta.require_list()?;

    let parser = syn::punctuated::Punctuated::<AttrItem, syn::Token![,]>::parse_terminated;
    let items: syn::punctuated::Punctuated<AttrItem, syn::Token![,]> =
        parser.parse2(meta_list.tokens.clone())?;

    for item in items {
        match item {
            AttrItem::KeyValue { key, value } => {
                attrs.push(AttrSpec {
                    key: classify_key(&key),
                    value: classify_value(&value),
                });
            }
            AttrItem::KeyBool { key, value } => {
                if value {
                    attrs.push(AttrSpec {
                        key: classify_key(&key),
                        value: AttrValue::Bool(true),
                    });
                }
            }
            AttrItem::KeyPath { key, path } => {
                attrs.push(AttrSpec {
                    key: classify_key(&key),
                    value: AttrValue::Path(path),
                });
            }
            AttrItem::KeySignalField { key, field } => {
                attrs.push(AttrSpec {
                    key: classify_key(&key),
                    value: AttrValue::SignalFieldBinding(field),
                });
            }
            AttrItem::KeyExpr { key, expr } => {
                attrs.push(AttrSpec {
                    key: classify_key(&key),
                    value: AttrValue::Expr(expr),
                });
            }
            AttrItem::BareKey { key } => {
                attrs.push(AttrSpec {
                    key: AttrKey::Literal(key),
                    value: AttrValue::Bool(true),
                });
            }
        }
    }

    Ok(attrs)
}

enum AttrItem {
    KeyValue { key: String, value: String },
    KeyBool { key: String, value: bool },
    KeyPath { key: String, path: syn::Path },
    KeySignalField { key: String, field: syn::Ident },
    KeyExpr { key: String, expr: syn::Expr },
    BareKey { key: String },
}

fn is_signal_field_binding_key(key: &str) -> bool {
    key == "data-bind" || key == "data_bind"
}

fn looks_like_field_name(ident: &Ident) -> bool {
    let s = ident.to_string();
    s.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
}

impl syn::parse::Parse for AttrItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = if input.peek(syn::LitStr) {
            let lit: syn::LitStr = input.parse()?;
            lit.value()
        } else {
            let ident: Ident = input.parse()?;
            normalize_attr_key(&ident.to_string())
        };

        if input.peek(syn::Token![=]) {
            input.parse::<syn::Token![=]>()?;

            if input.peek(syn::LitStr) {
                let lit: syn::LitStr = input.parse()?;
                return Ok(AttrItem::KeyValue {
                    key,
                    value: lit.value(),
                });
            } else if input.peek(syn::LitBool) {
                let lit: syn::LitBool = input.parse()?;
                return Ok(AttrItem::KeyBool {
                    key,
                    value: lit.value,
                });
            } else {
                let expr: syn::Expr = input.parse()?;

                if is_signal_field_binding_key(&key) {
                    if let syn::Expr::Path(expr_path) = &expr {
                        if let Some(ident) = expr_path.path.get_ident() {
                            if looks_like_field_name(ident) {
                                return Ok(AttrItem::KeySignalField {
                                    key,
                                    field: ident.clone(),
                                });
                            }
                        }
                    }
                }

                if let syn::Expr::Path(expr_path) = &expr {
                    if expr_path.qself.is_none() {
                        return Ok(AttrItem::KeyPath {
                            key,
                            path: expr_path.path.clone(),
                        });
                    }
                }

                return Ok(AttrItem::KeyExpr { key, expr });
            }
        } else {
            Ok(AttrItem::BareKey { key })
        }
    }
}

enum FieldAttrResult {
    IsAttr { rename: Option<String> },
    Attrs(Vec<AttrSpec>),
}

fn parse_field_attr_attribute(
    attr: &Attribute,
    _field_name: &Ident,
    _field_type: &Type,
) -> syn::Result<FieldAttrResult> {
    match &attr.meta {
        Meta::Path(_) => Ok(FieldAttrResult::IsAttr { rename: None }),
        Meta::List(list) => {
            if list.tokens.is_empty() {
                return Ok(FieldAttrResult::IsAttr { rename: None });
            }

            let mut rename = None;
            let mut attrs = Vec::new();

            let parser =
                syn::punctuated::Punctuated::<FieldAttrItem, syn::Token![,]>::parse_terminated;
            let items = parser.parse2(list.tokens.clone())?;

            for item in items {
                match item {
                    FieldAttrItem::Rename(name) => {
                        rename = Some(name);
                    }
                    FieldAttrItem::Attr(attr_item) => match attr_item {
                        AttrItem::KeyValue { key, value } => {
                            attrs.push(AttrSpec {
                                key: classify_key(&key),
                                value: classify_value(&value),
                            });
                        }
                        AttrItem::KeyBool { key, value } => {
                            if value {
                                attrs.push(AttrSpec {
                                    key: classify_key(&key),
                                    value: AttrValue::Bool(true),
                                });
                            }
                        }
                        AttrItem::KeyPath { key, path } => {
                            attrs.push(AttrSpec {
                                key: classify_key(&key),
                                value: AttrValue::Path(path),
                            });
                        }
                        AttrItem::KeySignalField { key, field } => {
                            attrs.push(AttrSpec {
                                key: classify_key(&key),
                                value: AttrValue::SignalFieldBinding(field),
                            });
                        }
                        AttrItem::BareKey { key } => {
                            attrs.push(AttrSpec {
                                key: AttrKey::Literal(key),
                                value: AttrValue::Bool(true),
                            });
                        }
                        AttrItem::KeyExpr { key, expr } => {
                            attrs.push(AttrSpec {
                                key: classify_key(&key),
                                value: AttrValue::Expr(expr),
                            });
                        }
                    },
                }
            }

            if rename.is_some() && attrs.is_empty() {
                Ok(FieldAttrResult::IsAttr { rename })
            } else if !attrs.is_empty() {
                Ok(FieldAttrResult::Attrs(attrs))
            } else {
                Ok(FieldAttrResult::IsAttr { rename: None })
            }
        }
        Meta::NameValue(_) => Err(syn::Error::new_spanned(
            attr,
            "unexpected name=value syntax",
        )),
    }
}

enum FieldAttrItem {
    Rename(String),
    Attr(AttrItem),
}

impl syn::parse::Parse for FieldAttrItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) {
            let ident: Ident = input.fork().parse()?;
            if ident == "name" && input.peek2(syn::Token![=]) {
                input.parse::<Ident>()?;
                input.parse::<syn::Token![=]>()?;
                let lit: syn::LitStr = input.parse()?;
                return Ok(FieldAttrItem::Rename(lit.value()));
            }
        }
        Ok(FieldAttrItem::Attr(input.parse()?))
    }
}

fn normalize_attr_key(key: &str) -> String {
    key.replace('_', "-")
}

fn has_interpolation(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                continue;
            }
            let mut ident = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '}' {
                    chars.next();
                    break;
                }
                ident.push(chars.next().unwrap());
            }
            if is_valid_identifier(&ident) {
                return true;
            }
        }
    }
    false
}

fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn classify_key(key: &str) -> AttrKey {
    if has_interpolation(key) {
        AttrKey::Interpolated(key.to_string())
    } else {
        AttrKey::Literal(key.to_string())
    }
}

fn classify_value(val: &str) -> AttrValue {
    if has_interpolation(val) {
        AttrValue::Interpolated(val.to_string())
    } else {
        AttrValue::Literal(val.to_string())
    }
}

pub fn is_bool_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "bool";
        }
    }
    false
}

pub fn is_vec_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Vec";
        }
    }
    false
}

pub fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

pub fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty())
}
