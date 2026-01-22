use crate::Element;

#[test]
fn struct_vec_string() {
    #[derive(Element)]
    #[element("div")]
    struct Hello {
        #[element]
        who: Vec<String>,
    }

    let hello = Hello {
        who: vec!["world".into(), "hello".into()],
    };

    let html = hello.to_string();

    assert_eq!(html, "<div>worldhello</div>");
}

#[test]
fn component_list() {
    #[derive(Element)]
    #[element("div")]
    struct Hello {
        #[element("ul")]
        who: Vec<WhoComponent>,
    }

    #[derive(Element)]
    #[element("li")]
    enum WhoComponent {
        World(String),
        Hello(String),
    }

    let hello = Hello {
        who: vec![
            WhoComponent::World("World".into()),
            WhoComponent::Hello("Hello".into()),
        ],
    };

    let html = hello.to_string();

    assert_eq!(html, "<div><ul><li>World</li><li>Hello</li></ul></div>");
}
