//! Kramdown block attribute parsing and representation.
//!
//! Kramdown-style attributes use the syntax `{:#id .class key="val"}` and
//! can appear on the line following a block element.  The [`Attributes`]
//! struct stores the parsed `id`, `classes`, and arbitrary key/value pairs
//! and can render them as an HTML attribute string via
//! [`Attributes::to_html_attr_string`].

use serde::Serialize;

/// Parsed Kramdown block attributes (`{:#id .class key="val"}`).
///
/// # Example
///
/// ```rust
/// use peisar_ast::tokens::Attributes;
///
/// let attrs = Attributes {
///     id: Some("intro".into()),
///     classes: Some(vec!["banner".into(), "wide".into()]),
///     attributes: Some(vec![("data-index".into(), "42".into())]),
/// };
/// assert_eq!(
///     attrs.to_html_attr_string(),
///     "id=\"intro\" class=\"banner wide\" data-index=\"42\""
/// );
/// ```
#[cfg_attr(feature = "napi", napi::napi(object))]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Attributes {
    /// HTML `id` attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// CSS class list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<Vec<String>>,
    /// Arbitrary key/value attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<(String, String)>>,
}

impl Attributes {
    /// Render the attributes as a space-delimited HTML attribute string.
    ///
    /// Special characters (`&`, `"`, `<`, `>`) are escaped.
    pub fn to_html_attr_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref id) = self.id {
            parts.push(format!("id=\"{}\"", escape_attr(id)));
        }
        if !self.classes.is_none() {
            // let classes = self.classes.clone().unwrap();
            parts.push(format!(
                "class=\"{}\"",
                escape_attr(&self.classes.clone().unwrap().join(" "))
            ));
        }
        for (k, v) in &self.attributes.clone().unwrap() {
            if k == "style" {
                parts.push(format!("style=\"{}\"", escape_attr(v)));
            }
            parts.push(format!("{}=\"{}\"", escape_attr(k), escape_attr(v)));
        }
        parts.join(" ")
    }
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
