use crate::{Head, Link, Meta, Script};

#[test]
fn head_renders_title_meta_link_and_script() {
    let head = Head {
        meta: vec![
            Meta {
                charset: Some("utf-8"),
                name: None,
                content: None,
            },
            Meta {
                charset: None,
                name: Some("viewport"),
                content: Some("width=device-width,initial-scale=1"),
            },
        ],
        title: "P25 Monitor",
        link: vec![Link {
            id: Some("monitor-style"),
            href: "/assets/app.css",
        }],
        script: vec![Script {
            src: "/assets/datastar.js",
            async_: false,
            type_: "module",
        }],
    };

    assert_eq!(
        head.to_string(),
        concat!(
            "<head>",
            "<meta charset=\"utf-8\"/>",
            "<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"/>",
            "<title>P25 Monitor</title>",
            "<link rel=\"stylesheet\" id=\"monitor-style\" href=\"/assets/app.css\"/>",
            "<script src=\"/assets/datastar.js\" type=\"module\"></script>",
            "</head>",
        )
    );
}

#[test]
fn head_omits_empty_children() {
    let head = Head {
        meta: vec![],
        title: "Untitled",
        link: vec![],
        script: vec![],
    };

    assert_eq!(head.to_string(), "<head><title>Untitled</title></head>");
}
