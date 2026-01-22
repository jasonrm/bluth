use crate::Element;
use std::fmt::Display;

#[test]
fn struct_fragment() {
    #[derive(Element)]
    struct Hello {
        #[element]
        who: Who,
    }

    #[derive(Element)]
    struct Who {
        #[element("div")]
        name: String,
    }

    let hello = Hello {
        who: Who {
            name: "world".into(),
        },
    };

    let html = hello.to_string();

    assert_eq!(html, "<div>world</div>");
}

#[test]
fn enum_fragment() {
    #[derive(Element)]
    #[element("div")]
    enum Hello {
        World(String),

        #[allow(unused)]
        UnusedWorld(String),
    }

    let hello = Hello::World("world".into());

    let html = hello.to_string();

    assert_eq!(html, "<div>world</div>");
}

#[test]
fn doctype_fragment() {
    #[derive(Element)]
    struct Document<T>
    where
        T: Display,
    {
        #[element]
        doctype: &'static str,

        #[element("html")]
        html: Vec<T>,
    }

    #[derive(Element)]
    #[element("div")]
    enum WhoComponent {
        World(String),
        Hello(String),
    }

    let hello = Document {
        doctype: "<!DOCTYPE html>",
        html: vec![
            WhoComponent::World("World".into()),
            WhoComponent::Hello("Hello".into()),
        ],
    };

    let html = hello.to_string();

    assert_eq!(
        html,
        "<!DOCTYPE html><html><div>World</div><div>Hello</div></html>"
    );
}
