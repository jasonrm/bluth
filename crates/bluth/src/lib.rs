pub use bluth_macros::{Element, Signal};

#[macro_export]
macro_rules! define_url {
    ($name:ident, $prefix:literal, $($param:ident: $ty:ty),+ $(,)?) => {
        #[derive(serde::Deserialize)]
        pub struct $name {
            $(pub $param: $ty),+
        }

        impl $name {
            pub const PATTERN: &'static str = concat!($prefix, $("/{", stringify!($param), "}"),+);

            pub fn new($($param: $ty),+) -> Self {
                Self { $($param),+ }
            }

            pub fn path(&self) -> String {
                let mut s = String::from($prefix);
                $(
                    s.push('/');
                    s.push_str(&self.$param.to_string());
                )+
                s
            }
        }
    };
}

#[cfg(test)]
mod tests;

use std::fmt::Display;

pub mod datastar;
pub mod html;
pub mod signal;

#[cfg(feature = "axum")]
pub mod extractor;

pub use signal::{OptDisplay, SignalEnum, SignalSelector, SignalValue};

#[cfg(feature = "axum")]
pub use extractor::{Signal as SignalExtractor, Signals};

#[derive(Element)]
pub struct Document<T>
where
    T: Display,
{
    #[element]
    doctype: &'static str,

    #[element]
    html: Html<T>,
}

impl<T> Document<T>
where
    T: Display,
{
    pub fn new(html: Html<T>) -> Self {
        Self {
            doctype: "<!doctype html>",
            html,
        }
    }
}

#[derive(Element)]
#[element("html")]
pub struct Html<T>
where
    T: Display,
{
    #[attr]
    pub lang: &'static str,

    #[element]
    pub head: Head,

    #[element]
    pub body: Body<T>,
}

#[derive(Element)]
#[element("body")]
pub struct Body<T>
where
    T: Display,
{
    #[attr]
    pub class: &'static str,

    #[element]
    pub children: Vec<T>,
}

#[derive(Element)]
#[element("head")]
pub struct Head {
    #[element]
    pub link: Vec<Link>,

    #[element]
    pub script: Vec<Script>,
}

#[derive(Element)]
#[element("link")]
#[attr(rel = "stylesheet")]
pub struct Link {
    #[attr]
    pub id: Option<&'static str>,

    #[attr]
    pub href: &'static str,
}

#[derive(Element)]
#[element("script")]
pub struct Script {
    #[attr]
    pub src: &'static str,

    #[attr(name = "async")]
    pub async_: bool,

    #[attr(name = "type")]
    pub type_: &'static str,
}
