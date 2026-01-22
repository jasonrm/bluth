use crate::Element;

#[test]
fn format_test() {
    #[derive(Element)]
    #[element("div")]
    struct Hello {
        #[element("ul")]
        who: Vec<WhoComponent>,
    }

    #[derive(Element)]
    #[element("li")]
    enum WhoComponent {
        World(u8),
        Hello(u16),
        #[format("{:.4}")]
        Float(f64),
        #[format("<{:.4}, {:.4}>")]
        Vec2(f64, f64),
    }

    let hello = Hello {
        who: vec![
            WhoComponent::World(12),
            WhoComponent::Hello(13),
            WhoComponent::Float(14.5),
            WhoComponent::Vec2(1.23456789, 2.3456789),
        ],
    };

    let html = hello.to_string();

    assert_eq!(
        html,
        "<div><ul><li>12</li><li>13</li><li>14.5000</li><li><1.2346, 2.3457></li></ul></div>"
    );
}

#[test]
fn format_strings() {
    #[derive(Element)]
    #[element("div")]
    #[format("{greeting} {name}!")]
    struct Hello {
        greeting: String,
        name: String,
    }

    let hello = Hello {
        greeting: "Hello".to_string(),
        name: "World".to_string(),
    };

    let html = hello.to_string();

    assert_eq!(html, "<div>Hello World!</div>");
}

#[test]
fn format_options() {
    #[derive(Element)]
    #[element("div")]
    #[format("{greeting} {name}!")]
    struct Hello {
        greeting: String,

        #[map_or("Unknown")]
        name: Option<f64>,
    }

    let hello = Hello {
        greeting: "Hello".to_string(),
        name: None,
    };

    let html = hello.to_string();

    assert_eq!(html, "<div>Hello Unknown!</div>");
}

#[test]
fn format_option_u64() {
    #[derive(Element)]
    #[element("tr")]
    struct TableRowItem {
        #[element("td")]
        name: String,

        #[element("td")]
        #[map_or("-")]
        cost: Option<u64>,

        #[element("td")]
        price: u64,

        #[element("td")]
        #[map_or("-")]
        #[format("{:.0}")]
        value: Option<f64>,
    }

    #[derive(Element)]
    struct TableItems {
        #[element("div")]
        #[attr(class = "grid")]
        items: Vec<TableRowItem>,
    }

    let hello = TableItems {
        items: vec![
            TableRowItem {
                name: "Item 1".to_string(),
                cost: Some(100),
                price: 200,
                value: Some(300.5),
            },
            TableRowItem {
                name: "Item 2".to_string(),
                cost: None,
                price: 300,
                value: None,
            },
        ],
    };

    let html = hello.to_string();

    assert_eq!(
        html,
        "<div class=\"grid\"><tr><td>Item 1</td><td>100</td><td>200</td><td>300</td></tr><tr><td>Item 2</td><td>-</td><td>300</td><td>-</td></tr></div>"
    );
}

#[test]
fn format_with_expression_args() {
    struct Point {
        x: f64,
        y: f64,
    }

    impl Point {
        fn x_str(&self) -> String {
            format!("{:.2}", self.x)
        }
        fn y_str(&self) -> String {
            format!("{:.2}", self.y)
        }
    }

    #[derive(Element)]
    #[element("span")]
    #[format("({}, {})", point.x_str(), point.y_str())]
    struct PointDisplay {
        point: Point,
    }

    let display = PointDisplay {
        point: Point {
            x: 3.14159,
            y: 2.71828,
        },
    };
    let html = display.to_string();

    assert_eq!(html, "<span>(3.14, 2.72)</span>");
}

#[test]
fn format_empty() {
    #[derive(Element)]
    struct Hello {
        #[element("div")]
        #[attr(id = "test")]
        greeting: (),
    }

    let hello = Hello { greeting: () };

    let html = hello.to_string();

    assert_eq!(html, "<div id=\"test\"></div>");
}
