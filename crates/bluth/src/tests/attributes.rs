use crate::Element;

#[test]
fn attrs() {
    #[derive(Element)]
    #[element("div")]
    #[attr(hx_target = "#target", hx_post = "/api/{greeting}/{name}")]
    #[attr(hx_swap = "innerHTML")]
    struct Hello {
        greeting: String,
        name: String,
    }

    let hello = Hello {
        greeting: "Hello".to_string(),
        name: "World".to_string(),
    };

    let html = hello.to_string();

    assert_eq!(
        html,
        "<div hx-target=\"#target\" hx-post=\"/api/Hello/World\" hx-swap=\"innerHTML\"></div>"
    );
}

#[test]
fn attr_boolean() {
    #[derive(Element)]
    #[element("div")]
    #[attr(data_hello = false, data_world = true)]
    struct Hello {}

    let hello = Hello {};

    let html = hello.to_string();

    assert_eq!(html, "<div data-world></div>");
}

#[test]
fn attrs_reserved() {
    #[derive(Element)]
    #[element("button")]
    #[attr("type" = "button", class = "btn btn-primary", id = "btn-id")]
    struct Hello {
        #[element]
        name: String,
    }

    let hello = Hello {
        name: "World".to_string(),
    };

    let html = hello.to_string();

    assert_eq!(
        html,
        "<button type=\"button\" class=\"btn btn-primary\" id=\"btn-id\">World</button>"
    );
}

#[test]
fn field_attr() {
    #[derive(Element)]
    #[element("input")]
    struct Hello {
        #[attr]
        value: String,

        #[attr]
        disabled: bool,
    }

    let hello = Hello {
        value: "World".to_string(),
        disabled: true,
    };

    let html = hello.to_string();

    assert_eq!(html, "<input value=\"World\" disabled/>");
}

#[test]
fn field_attr_rename() {
    #[derive(Element)]
    #[element("script")]
    pub struct Script {
        #[attr]
        pub src: &'static str,

        #[attr(name = "async")]
        pub async_: bool,
    }

    let hello = Script {
        src: "/script.js",
        async_: true,
    };

    let html = hello.to_string();

    assert_eq!(html, "<script src=\"/script.js\" async></script>");
}

#[test]
fn attr_double_brace_escaping() {
    #[derive(Element)]
    #[element("form")]
    #[attr(
        "data-on:submit" = "@post('{action_url}', {{contentType: 'form'}})",
        class = "space-y-4"
    )]
    struct TestForm {
        action_url: String,
    }

    let form = TestForm {
        action_url: "/api/submit".to_string(),
    };

    let html = form.to_string();

    assert_eq!(
        html,
        "<form data-on:submit=\"@post('/api/submit', {contentType: 'form'})\" class=\"space-y-4\"></form>"
    );
}

#[test]
fn attr_only_double_braces_no_interpolation() {
    #[derive(Element)]
    #[element("div")]
    #[attr("data-config" = "{{key: 'value'}}")]
    struct Config {}

    let config = Config {};

    let html = config.to_string();

    assert_eq!(html, "<div data-config=\"{key: 'value'}\"></div>");
}
