use heck::ToLowerCamelCase;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, parse_macro_input};

mod attributes;
mod codegen;

use attributes::ElementSpec;
use codegen::{generate_enum_render, generate_struct_render};

fn get_bluth_crate() -> proc_macro2::TokenStream {
    match crate_name("bluth") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::bluth),
    }
}

#[proc_macro_derive(Element, attributes(element, format, attr, map_or))]
pub fn derive_element(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_element_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_element_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let spec = ElementSpec::from_attrs(&input.attrs)?;
    let bluth_crate = get_bluth_crate();

    let render_body = match &input.data {
        Data::Struct(data) => generate_struct_render(data, &spec, &bluth_crate)?,
        Data::Enum(data) => generate_enum_render(name, data, &spec, &bluth_crate)?,
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "Element can only be derived for structs and enums",
            ));
        }
    };

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #render_body
                Ok(())
            }
        }
    })
}

#[proc_macro_derive(Signal, attributes(signal))]
pub fn derive_signal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_signal_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_signal_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    let Data::Enum(enum_data) = &input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Signal can only be derived for enums",
        ));
    };

    generate_signal_enum(name, enum_data, &input.vis)
}

struct VariantInfo {
    variant_name: syn::Ident,
    signal_name: String,
    field_type: syn::Type,
}

fn parse_variant(variant: &syn::Variant) -> syn::Result<VariantInfo> {
    let variant_name = variant.ident.clone();

    let Fields::Unnamed(fields) = &variant.fields else {
        return Err(syn::Error::new_spanned(
            variant,
            "Signal variants must have exactly one unnamed field",
        ));
    };

    if fields.unnamed.len() != 1 {
        return Err(syn::Error::new_spanned(
            variant,
            "Signal variants must have exactly one unnamed field",
        ));
    }

    let field_type = fields.unnamed.first().expect("checked len above").ty.clone();

    let signal_name = variant
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("signal"))
        .and_then(|attr| {
            attr.parse_args_with(|input: syn::parse::ParseStream| {
                let ident: syn::Ident = input.parse()?;
                if ident != "name" {
                    return Err(syn::Error::new_spanned(ident, "expected `name`"));
                }
                input.parse::<syn::Token![=]>()?;
                let lit: syn::LitStr = input.parse()?;
                Ok(lit.value())
            })
            .ok()
        })
        .unwrap_or_else(|| variant_name.to_string().to_lower_camel_case());

    Ok(VariantInfo {
        variant_name,
        signal_name,
        field_type,
    })
}

fn generate_signal_enum(
    enum_name: &syn::Ident,
    data: &DataEnum,
    vis: &syn::Visibility,
) -> syn::Result<proc_macro2::TokenStream> {
    let bluth = get_bluth_crate();

    let variants: Vec<VariantInfo> = data
        .variants
        .iter()
        .map(parse_variant)
        .collect::<syn::Result<_>>()?;

    let selector_structs: Vec<_> = variants
        .iter()
        .map(|v| {
            let selector_name = &v.variant_name;
            quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                #vis struct #selector_name;
            }
        })
        .collect();

    let selector_impls: Vec<_> = variants
        .iter()
        .map(|v| {
            let selector_name = &v.variant_name;
            let signal_name = &v.signal_name;
            let field_type = &v.field_type;

            quote! {
                impl #bluth::SignalSelector for #selector_name {
                    type Value = #field_type;
                    type Enum = #enum_name;

                    const NAME: &'static str = #signal_name;

                    fn extract(value: &#enum_name) -> ::core::option::Option<&Self::Value> {
                        match value {
                            #enum_name::#selector_name(v) => ::core::option::Option::Some(v),
                            _ => ::core::option::Option::None,
                        }
                    }

                    fn into_inner(value: #enum_name) -> ::core::option::Option<Self::Value> {
                        match value {
                            #enum_name::#selector_name(v) => ::core::option::Option::Some(v),
                            _ => ::core::option::Option::None,
                        }
                    }

                    fn wrap(value: Self::Value) -> #enum_name {
                        #enum_name::#selector_name(value)
                    }
                }

                impl ::core::convert::AsRef<str> for #selector_name {
                    fn as_ref(&self) -> &str {
                        <#selector_name as #bluth::SignalSelector>::NAME
                    }
                }
            }
        })
        .collect();

    let signal_name_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant_name = &v.variant_name;
            let signal_name = &v.signal_name;
            quote! {
                Self::#variant_name(_) => #signal_name,
            }
        })
        .collect();

    let to_json_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant_name = &v.variant_name;
            quote! {
                Self::#variant_name(v) => ::serde_json::to_value(v).unwrap_or(::serde_json::Value::Null),
            }
        })
        .collect();

    let serialize_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant_name = &v.variant_name;
            let signal_name = &v.signal_name;
            quote! {
                Self::#variant_name(v) => map.serialize_entry(#signal_name, v)?,
            }
        })
        .collect();

    let signal_enum_impl = quote! {
        impl #bluth::SignalEnum for #enum_name {
            fn signal_name(&self) -> &'static str {
                match self {
                    #(#signal_name_arms)*
                }
            }

            fn to_json_value(&self) -> ::serde_json::Value {
                match self {
                    #(#to_json_arms)*
                }
            }
        }
    };

    let serialize_impl = quote! {
        impl ::serde::Serialize for #enum_name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                use ::serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(::core::option::Option::Some(1))?;
                match self {
                    #(#serialize_arms)*
                }
                map.end()
            }
        }
    };

    let clone_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant_name = &v.variant_name;
            quote! {
                Self::#variant_name(v) => Self::#variant_name(::core::clone::Clone::clone(v)),
            }
        })
        .collect();

    let clone_impl = quote! {
        #[automatically_derived]
        impl ::core::clone::Clone for #enum_name {
            fn clone(&self) -> Self {
                match self {
                    #(#clone_arms)*
                }
            }
        }
    };

    let debug_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant_name = &v.variant_name;
            let variant_str = variant_name.to_string();
            quote! {
                Self::#variant_name(v) => f.debug_tuple(#variant_str).field(v).finish(),
            }
        })
        .collect();

    let debug_impl = quote! {
        #[automatically_derived]
        impl ::core::fmt::Debug for #enum_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(#debug_arms)*
                }
            }
        }
    };

    Ok(quote! {
        #(#selector_structs)*

        #(#selector_impls)*

        #signal_enum_impl

        #serialize_impl

        #clone_impl

        #debug_impl
    })
}
