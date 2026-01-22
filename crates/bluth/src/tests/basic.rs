use crate::Element;

#[test]
fn struct_to_html() {
    #[derive(Element)]
    #[element("div")]
    struct Hello {
        #[element("span")]
        who: String,

        #[allow(unused)]
        ignored_data: u64,
    }

    let hello = Hello {
        who: "world".into(),
        ignored_data: 123,
    };

    let html = hello.to_string();

    assert_eq!(html, "<div><span>world</span></div>");
}

#[test]
fn enum_to_html() {
    #[derive(Element)]
    #[element("div")]
    enum Hello {
        #[element("span")]
        World(String),

        #[element("span")]
        #[allow(unused)]
        UnusedWorld(String),
    }

    let hello = Hello::World("world".into());

    let html = hello.to_string();

    assert_eq!(html, "<div><span>world</span></div>");
}
