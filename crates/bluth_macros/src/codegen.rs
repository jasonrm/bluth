use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::{DataEnum, DataStruct, Fields, GenericArgument, Ident, PathArguments, Type};

use crate::attributes::{
    AttrKey, AttrSpec, AttrValue, ElementSpec, FieldSpec, FormatSpec, is_bool_type, is_option_type,
    is_unit_type, is_vec_type,
};

pub struct SignalFieldInfo {
    pub selector_type: syn::Path,
}

fn extract_signal_value_type(ty: &Type) -> Option<syn::Path> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "SignalValue" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let GenericArgument::Type(Type::Path(inner_path)) = args.args.first()? else {
        return None;
    };
    Some(inner_path.path.clone())
}

fn collect_signal_fields(fields: &Fields) -> HashMap<String, SignalFieldInfo> {
    let Fields::Named(named) = fields else {
        return HashMap::new();
    };
    named
        .named
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;
            let selector_type = extract_signal_value_type(&field.ty)?;
            Some((field_name.to_string(), SignalFieldInfo { selector_type }))
        })
        .collect()
}

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn is_void_element(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag)
}

pub fn generate_struct_render(
    data: &DataStruct,
    spec: &ElementSpec,
    bluth_crate: &TokenStream,
) -> syn::Result<TokenStream> {
    let signal_fields = collect_signal_fields(&data.fields);

    let field_renders = match &data.fields {
        Fields::Named(fields) => {
            if let Some(ref format_spec) = spec.format {
                generate_formatted_struct_render(fields, format_spec)
            } else {
                generate_named_field_renders(fields, &signal_fields, bluth_crate)?
            }
        }
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let field = fields.unnamed.first().unwrap();
            generate_tuple_struct_render(&field.ty, &spec.map_or)
        }
        Fields::Unnamed(_) | Fields::Unit => TokenStream::new(),
    };

    let field_attrs = collect_field_attrs(&data.fields)?;

    Ok(wrap_with_tag(
        &field_renders,
        spec,
        &field_attrs,
        &signal_fields,
        bluth_crate,
    ))
}

fn collect_field_attrs(fields: &Fields) -> syn::Result<Vec<(Ident, syn::Type, String)>> {
    let mut result = Vec::new();

    if let Fields::Named(named) = fields {
        for field in &named.named {
            let field_name = field.ident.as_ref().unwrap();
            let field_spec = FieldSpec::from_attrs(&field.attrs, field_name, &field.ty)?;

            if field_spec.is_attr {
                let attr_name = field_spec
                    .attr_rename
                    .unwrap_or_else(|| field_name.to_string().replace('_', "-"));
                result.push((field_name.clone(), field.ty.clone(), attr_name));
            }
        }
    }

    Ok(result)
}

fn generate_formatted_struct_render(
    fields: &syn::FieldsNamed,
    format_spec: &FormatSpec,
) -> TokenStream {
    let format_string = &format_spec.format_string;

    if let Some(ref args) = format_spec.args {
        let field_names: HashSet<String> = fields
            .named
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect();
        let transformed_args = prefix_self_to_idents(args.clone(), &field_names);
        quote! {
            write!(f, #format_string, #transformed_args)?;
        }
    } else {
        let field_bindings = fields.named.iter().filter_map(|field| {
            let field_name = field.ident.as_ref()?;
            let field_spec = FieldSpec::from_attrs(&field.attrs, field_name, &field.ty).ok()?;

            if let Some(default_value) = field_spec.map_or {
                if is_option_type(&field.ty) {
                    Some(quote! { #field_name = self.#field_name.as_ref().map(|v| v.to_string()).unwrap_or_else(|| #default_value.to_string()) })
                } else {
                    Some(quote! { #field_name = self.#field_name })
                }
            } else {
                Some(quote! { #field_name = self.#field_name })
            }
        });

        quote! {
            write!(f, #format_string, #(#field_bindings),*)?;
        }
    }
}

fn prefix_self_to_idents(tokens: TokenStream, field_names: &HashSet<String>) -> TokenStream {
    let mut result = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ref ident) if field_names.contains(&ident.to_string()) => {
                let self_ident = proc_macro2::Ident::new("self", ident.span());
                result.push(TokenTree::Ident(self_ident));
                result.push(TokenTree::Punct(proc_macro2::Punct::new(
                    '.',
                    proc_macro2::Spacing::Alone,
                )));
                result.push(token);
            }
            TokenTree::Group(group) => {
                let inner = prefix_self_to_idents(group.stream(), field_names);
                let new_group = proc_macro2::Group::new(group.delimiter(), inner);
                result.push(TokenTree::Group(new_group));
            }
            _ => {
                result.push(token);
            }
        }
    }

    result.into_iter().collect()
}

fn generate_named_field_renders(
    fields: &syn::FieldsNamed,
    signal_fields: &HashMap<String, SignalFieldInfo>,
    bluth_crate: &TokenStream,
) -> syn::Result<TokenStream> {
    let mut renders = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let field_spec = FieldSpec::from_attrs(&field.attrs, field_name, field_type)?;

        if field_spec.is_attr {
            continue;
        }

        if !field_spec.should_render {
            continue;
        }

        let is_vec = is_vec_type(field_type);
        let is_option = is_option_type(field_type);
        let is_unit = is_unit_type(field_type);

        let content = if is_unit {
            quote! {}
        } else if is_vec {
            quote! {
                for item in &self.#field_name {
                    write!(f, "{}", item)?;
                }
            }
        } else if is_option {
            if let Some(ref default_val) = field_spec.map_or {
                if let Some(ref format_spec) = field_spec.format {
                    let fmt_str = &format_spec.format_string;
                    if let Some(ref args) = format_spec.args {
                        quote! {
                            match &self.#field_name {
                                Some(_v) => write!(f, #fmt_str, #args)?,
                                None => write!(f, "{}", #default_val)?,
                            }
                        }
                    } else {
                        quote! {
                            match &self.#field_name {
                                Some(v) => write!(f, #fmt_str, v)?,
                                None => write!(f, "{}", #default_val)?,
                            }
                        }
                    }
                } else {
                    quote! {
                        match &self.#field_name {
                            Some(v) => write!(f, "{}", v)?,
                            None => write!(f, "{}", #default_val)?,
                        }
                    }
                }
            } else {
                quote! {
                    if let Some(ref v) = self.#field_name {
                        write!(f, "{}", v)?;
                    }
                }
            }
        } else if let Some(ref format_spec) = field_spec.format {
            let fmt_str = &format_spec.format_string;
            if let Some(ref args) = format_spec.args {
                quote! {
                    write!(f, #fmt_str, #args)?;
                }
            } else {
                quote! {
                    write!(f, #fmt_str, self.#field_name)?;
                }
            }
        } else {
            quote! {
                write!(f, "{}", self.#field_name)?;
            }
        };

        let render = if let Some(ref tag) = field_spec.tag {
            let is_void = is_void_element(tag);
            let attr_code = emit_attrs(&field_spec.attrs, true, signal_fields, bluth_crate);

            if is_void {
                quote! {
                    write!(f, "<{}", #tag)?;
                    #attr_code
                    write!(f, "/>")?;
                }
            } else {
                quote! {
                    write!(f, "<{}", #tag)?;
                    #attr_code
                    write!(f, ">")?;
                    #content
                    write!(f, "</{}>", #tag)?;
                }
            }
        } else {
            content
        };

        if is_unit {
            renders.push(quote! { let _ = &self.#field_name; });
        }

        renders.push(render);
    }

    Ok(quote! { #(#renders)* })
}

fn generate_tuple_struct_render(
    field_type: &syn::Type,
    map_or_value: &Option<String>,
) -> TokenStream {
    if is_option_type(field_type) {
        if let Some(default_value) = map_or_value {
            quote! {
                match &self.0 {
                    Some(v) => write!(f, "{}", v)?,
                    None => write!(f, "{}", #default_value)?,
                }
            }
        } else {
            quote! {
                if let Some(ref v) = self.0 {
                    write!(f, "{}", v)?;
                }
            }
        }
    } else {
        quote! {
            write!(f, "{}", self.0)?;
        }
    }
}

pub fn generate_enum_render(
    name: &Ident,
    data: &DataEnum,
    spec: &ElementSpec,
    _bluth_crate: &TokenStream,
) -> syn::Result<TokenStream> {
    let enum_tag = spec
        .tag
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(name, "Enum requires #[element(\"tag\")]"))?;

    let variant_matches: Vec<_> = data
        .variants
        .iter()
        .map(|variant| generate_variant_match(name, variant, enum_tag))
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        match self {
            #(#variant_matches)*
        }
    })
}

fn generate_variant_match(
    enum_name: &Ident,
    variant: &syn::Variant,
    enum_tag: &str,
) -> syn::Result<TokenStream> {
    let variant_name = &variant.ident;
    let variant_spec = FieldSpec::from_attrs(&variant.attrs, variant_name, &syn::parse_quote!(()))?;

    match &variant.fields {
        Fields::Unnamed(fields) if !fields.unnamed.is_empty() => Ok(generate_tuple_variant(
            enum_name,
            variant_name,
            variant_spec.tag.as_deref(),
            enum_tag,
            fields.unnamed.len(),
            variant_spec.format.as_ref(),
        )),
        Fields::Unit => Ok(generate_unit_variant(enum_name, variant_name, enum_tag)),
        _ => Err(syn::Error::new_spanned(
            variant,
            "Only unit variants and tuple variants are supported",
        )),
    }
}

fn generate_tuple_variant(
    enum_name: &Ident,
    variant_name: &Ident,
    variant_tag: Option<&str>,
    enum_tag: &str,
    field_count: usize,
    format_spec: Option<&FormatSpec>,
) -> TokenStream {
    let open_enum = format!("<{}>", enum_tag);
    let close_enum = format!("</{}>", enum_tag);

    let field_bindings: Vec<_> = (0..field_count)
        .map(|i| syn::Ident::new(&format!("field{}", i), proc_macro2::Span::call_site()))
        .collect();

    let pattern = if field_count == 1 {
        let field = &field_bindings[0];
        quote! { #enum_name::#variant_name(#field) }
    } else {
        quote! { #enum_name::#variant_name(#(#field_bindings),*) }
    };

    let content = if let Some(spec) = format_spec {
        let fmt_str = &spec.format_string;
        if let Some(ref args) = spec.args {
            quote! {
                write!(f, #fmt_str, #args)?;
            }
        } else {
            quote! {
                write!(f, #fmt_str, #(#field_bindings),*)?;
            }
        }
    } else if field_count == 1 {
        let field = &field_bindings[0];
        quote! {
            write!(f, "{}", #field)?;
        }
    } else {
        quote! {
            #(write!(f, "{}", #field_bindings)?;)*
        }
    };

    match variant_tag {
        Some(tag) => {
            let open_variant = format!("<{}>", tag);
            let close_variant = format!("</{}>", tag);
            quote! {
                #pattern => {
                    write!(f, "{}", #open_enum)?;
                    write!(f, "{}", #open_variant)?;
                    #content
                    write!(f, "{}", #close_variant)?;
                    write!(f, "{}", #close_enum)?;
                }
            }
        }
        None => {
            quote! {
                #pattern => {
                    write!(f, "{}", #open_enum)?;
                    #content
                    write!(f, "{}", #close_enum)?;
                }
            }
        }
    }
}

fn generate_unit_variant(enum_name: &Ident, variant_name: &Ident, enum_tag: &str) -> TokenStream {
    let open_tag = format!("<{}>", enum_tag);
    let close_tag = format!("</{}>", enum_tag);
    quote! {
        #enum_name::#variant_name => {
            write!(f, "{}", #open_tag)?;
            write!(f, "{}", #close_tag)?;
        }
    }
}

fn wrap_with_tag(
    content: &TokenStream,
    spec: &ElementSpec,
    field_attrs: &[(Ident, syn::Type, String)],
    signal_fields: &HashMap<String, SignalFieldInfo>,
    bluth_crate: &TokenStream,
) -> TokenStream {
    let Some(ref tag_name) = spec.tag else {
        return content.clone();
    };

    let is_void = is_void_element(tag_name);
    let attr_code = emit_attrs(&spec.attrs, true, signal_fields, bluth_crate);

    let field_attr_code: Vec<_> = field_attrs
        .iter()
        .map(|(field_name, field_type, attr_name)| {
            if is_bool_type(field_type) {
                quote! {
                    if self.#field_name {
                        write!(f, " {}", #attr_name)?;
                    }
                }
            } else if is_option_type(field_type) {
                quote! {
                    if let Some(ref v) = self.#field_name {
                        write!(f, " {}=\"{}\"", #attr_name, #bluth_crate::html::escape_attr(v))?;
                    }
                }
            } else {
                quote! {
                    write!(f, " {}=\"{}\"", #attr_name, #bluth_crate::html::escape_attr(&self.#field_name))?;
                }
            }
        })
        .collect();

    if spec.attrs.is_empty() && field_attrs.is_empty() {
        if is_void {
            let full_tag = format!("<{}/>", tag_name);
            return quote! {
                write!(f, "{}", #full_tag)?;
            };
        } else {
            let open = format!("<{}>", tag_name);
            let close = format!("</{}>", tag_name);
            return quote! {
                write!(f, "{}", #open)?;
                #content
                write!(f, "{}", #close)?;
            };
        }
    }

    let close_tag = format!("</{}>", tag_name);

    if is_void {
        quote! {
            write!(f, "<{}", #tag_name)?;
            #attr_code
            #(#field_attr_code)*
            write!(f, "/>")?;
        }
    } else {
        quote! {
            write!(f, "<{}", #tag_name)?;
            #attr_code
            #(#field_attr_code)*
            write!(f, ">")?;
            #content
            write!(f, "{}", #close_tag)?;
        }
    }
}

fn emit_attrs(
    attrs: &[AttrSpec],
    use_self: bool,
    signal_fields: &HashMap<String, SignalFieldInfo>,
    bluth_crate: &TokenStream,
) -> TokenStream {
    let attr_writes: Vec<_> = attrs
        .iter()
        .map(|attr| emit_single_attr(attr, use_self, signal_fields, bluth_crate))
        .collect();

    quote! { #(#attr_writes)* }
}

fn emit_single_attr(
    attr: &AttrSpec,
    use_self: bool,
    signal_fields: &HashMap<String, SignalFieldInfo>,
    bluth_crate: &TokenStream,
) -> TokenStream {
    let key_expr = match &attr.key {
        AttrKey::Literal(k) => quote! { #k },
        AttrKey::Interpolated(k) => interpolate(k, use_self),
    };

    match &attr.value {
        AttrValue::Literal(v) => {
            let escaped = escape_attr_str(v);
            quote! {
                write!(f, " {}=\"{}\"", #key_expr, #escaped)?;
            }
        }
        AttrValue::Interpolated(v) => {
            let val_expr = interpolate(v, use_self);
            quote! {
                write!(f, " {}=\"{}\"", #key_expr, #bluth_crate::html::escape_attr(#val_expr))?;
            }
        }
        AttrValue::Bool(true) => {
            quote! {
                write!(f, " {}", #key_expr)?;
            }
        }
        AttrValue::Bool(false) => {
            quote! {}
        }
        AttrValue::Path(path) => {
            quote! {
                write!(f, " {}=\"{}\"", #key_expr, #bluth_crate::html::escape_attr(<#path as ::core::convert::AsRef<str>>::as_ref(&#path)))?;
            }
        }
        AttrValue::SignalFieldBinding(field_ident) => {
            let field_name = field_ident.to_string();
            if let Some(signal_info) = signal_fields.get(&field_name) {
                let selector_type = &signal_info.selector_type;
                quote! {
                    let _ = &self.#field_ident;
                    write!(f, " {}=\"{}\"", #key_expr, <#selector_type as #bluth_crate::SignalSelector>::NAME)?;
                }
            } else {
                let err_msg = format!(
                    "Field '{}' is not a SignalValue<T> type. Use data_bind = SignalType for non-field bindings.",
                    field_name
                );
                quote! {
                    compile_error!(#err_msg);
                }
            }
        }
        AttrValue::Expr(expr) => {
            quote! {
                write!(f, " {}=\"{}\"", #key_expr, #bluth_crate::html::escape_attr(#expr))?;
            }
        }
    }
}

fn unescape_double_braces(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' && chars.peek() == Some(&'{') {
            chars.next();
            result.push('{');
        } else if ch == '}' && chars.peek() == Some(&'}') {
            chars.next();
            result.push('}');
        } else {
            result.push(ch);
        }
    }
    result
}

fn escape_attr_str(value: &str) -> String {
    let unescaped = unescape_double_braces(value);
    let mut result = String::with_capacity(unescaped.len());
    for ch in unescaped.chars() {
        match ch {
            '"' => result.push_str("&quot;"),
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(ch),
        }
    }
    result
}

fn interpolate(template: &str, use_self: bool) -> TokenStream {
    let mut format_parts = Vec::new();
    let mut value_parts: Vec<TokenStream> = Vec::new();
    let mut current_literal = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                current_literal.push_str("{{");
                continue;
            }

            if !current_literal.is_empty() {
                format_parts.push(current_literal.clone());
                current_literal.clear();
            }

            let mut field_name = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '}' {
                    chars.next();
                    break;
                }
                chars.next();
                field_name.push(next_ch);
            }

            let field_ident = syn::Ident::new(&field_name, proc_macro2::Span::call_site());
            format_parts.push("{}".to_string());

            if use_self {
                value_parts.push(quote! { &self.#field_ident });
            } else {
                value_parts.push(quote! { &#field_ident });
            }
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                chars.next();
                current_literal.push_str("}}");
                continue;
            }
            current_literal.push(ch);
        } else {
            current_literal.push(ch);
        }
    }

    if !current_literal.is_empty() {
        format_parts.push(current_literal);
    }

    let format_string = format_parts.join("");

    if value_parts.is_empty() {
        quote! { #format_string }
    } else {
        quote! { format!(#format_string, #(#value_parts),*) }
    }
}
