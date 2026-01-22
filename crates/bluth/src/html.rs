use std::fmt::{self, Display, Write};

pub struct EscapedAttr<T>(pub T);

impl<T: Display> Display for EscapedAttr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.0.to_string();
        for ch in value.chars() {
            match ch {
                '"' => f.write_str("&quot;")?,
                '&' => f.write_str("&amp;")?,
                '<' => f.write_str("&lt;")?,
                '>' => f.write_str("&gt;")?,
                _ => f.write_char(ch)?,
            }
        }
        Ok(())
    }
}

pub fn escape_attr<T: Display>(value: T) -> EscapedAttr<T> {
    EscapedAttr(value)
}

pub fn escape_attr_str(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for ch in value.chars() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_double_quotes() {
        let input = r#"hello "world""#;
        let escaped = escape_attr_str(input);
        assert_eq!(escaped, "hello &quot;world&quot;");
    }

    #[test]
    fn escape_ampersand() {
        let input = "foo & bar";
        let escaped = escape_attr_str(input);
        assert_eq!(escaped, "foo &amp; bar");
    }

    #[test]
    fn escape_angle_brackets() {
        let input = "<script>alert('xss')</script>";
        let escaped = escape_attr_str(input);
        assert_eq!(escaped, "&lt;script&gt;alert('xss')&lt;/script&gt;");
    }

    #[test]
    fn escape_mixed() {
        let input = r#"x = "a < b && c > d""#;
        let escaped = escape_attr_str(input);
        assert_eq!(escaped, "x = &quot;a &lt; b &amp;&amp; c &gt; d&quot;");
    }

    #[test]
    fn no_escape_needed() {
        let input = "hello world 123";
        let escaped = escape_attr_str(input);
        assert_eq!(escaped, "hello world 123");
    }

    #[test]
    fn escaped_attr_display() {
        let value = r#"say "hi""#;
        let escaped = format!("{}", escape_attr(value));
        assert_eq!(escaped, "say &quot;hi&quot;");
    }

    #[test]
    fn escaped_attr_with_number() {
        let value = 42;
        let escaped = format!("{}", escape_attr(value));
        assert_eq!(escaped, "42");
    }
}
