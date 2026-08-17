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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_double_quotes() {
        let escaped = EscapedAttr(r#"hello "world""#).to_string();
        assert_eq!(escaped, "hello &quot;world&quot;");
    }

    #[test]
    fn escape_ampersand() {
        let escaped = EscapedAttr("foo & bar").to_string();
        assert_eq!(escaped, "foo &amp; bar");
    }

    #[test]
    fn escape_angle_brackets() {
        let escaped = EscapedAttr("<script>alert('xss')</script>").to_string();
        assert_eq!(escaped, "&lt;script&gt;alert('xss')&lt;/script&gt;");
    }

    #[test]
    fn escape_mixed() {
        let escaped = EscapedAttr(r#"x = "a < b && c > d""#).to_string();
        assert_eq!(escaped, "x = &quot;a &lt; b &amp;&amp; c &gt; d&quot;");
    }

    #[test]
    fn no_escape_needed() {
        let escaped = EscapedAttr("hello world 123").to_string();
        assert_eq!(escaped, "hello world 123");
    }

    #[test]
    fn escaped_attr_display() {
        let escaped = EscapedAttr(r#"say "hi""#).to_string();
        assert_eq!(escaped, "say &quot;hi&quot;");
    }

    #[test]
    fn escaped_attr_with_number() {
        let escaped = EscapedAttr(42).to_string();
        assert_eq!(escaped, "42");
    }
}
