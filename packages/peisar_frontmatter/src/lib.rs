use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[cfg(feature = "napi")]
use napi_derive::napi;

/// Parsed Markdown content that may include YAML front matter.
///
/// `pure_markdown_content` contains the Markdown body without the front matter block,
/// while `yaml_data` stores the deserialized YAML payload when present.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult<T> {
    pure_markdown_content: String,
    yaml_data: Option<T>,
}

impl<T> ParseResult<T> {
    /// Returns the deserialized YAML front matter, if one was present.
    pub fn yaml_data(&self) -> Option<&T> {
        self.yaml_data.as_ref()
    }

    /// Returns the Markdown body with the YAML front matter removed.
    pub fn pure_markdown_content(&self) -> &str {
        &self.pure_markdown_content
    }

    /// Consume the ParseResult and return the owned Markdown content and owned
    /// optional YAML front matter in one allocation. This is useful for callers
    /// that need to take ownership of the parsed frontmatter value.
    pub fn into_parts(self) -> (String, Option<T>) {
        (self.pure_markdown_content, self.yaml_data)
    }
}

/// Parses a Markdown document with optional YAML front matter.
///
/// The function accepts content beginning with a `---` block and extracts the YAML
/// header until the closing `---` delimiter. The remainder of the document is returned
/// as plain Markdown content with leading whitespace trimmed.
///
/// # Examples
///
/// ```rust
/// use peisar_frontmatter::parse_markdown_frontmatter;
///
/// #[derive(serde::Deserialize)]
/// struct Frontmatter {
///     title: String,
/// }
///
/// let content = "---\ntitle: Hello\n---\n# Welcome\n";
/// let parsed = parse_markdown_frontmatter::<Frontmatter>(content).unwrap();
///
/// assert_eq!(parsed.yaml_data().unwrap().title, "Hello");
/// assert_eq!(parsed.pure_markdown_content(), "# Welcome\n");
/// ```
pub fn parse_markdown_frontmatter<T>(content: &str) -> Result<ParseResult<T>, String>
where
    T: DeserializeOwned,
{
    let trimmed = content.trim_start();
    let has_frontmatter_start = trimmed.starts_with("---\n") || trimmed.starts_with("---\r\n");
    let offset = if trimmed.starts_with("---\r\n") { 5 } else { 4 };
    let rest = &trimmed[offset..];
    let mut yaml_data: Option<T> = None;
    let mut pure_markdown_content: String = content.to_string();
    if has_frontmatter_start {
        if let Some(end_index) = rest.find("\n---") {
            let yaml_str = &rest[..end_index];
            let md_start_offset = end_index + 4;
            pure_markdown_content = rest[md_start_offset..].trim_start().to_string();

            // Parse into the generic type T
            yaml_data = serde_yaml::from_str(yaml_str)
                .map_err(|e| format!("Failed to parse YAML front matter: {}", e))?;
        }
    }
    Ok(ParseResult {
        yaml_data,
        pure_markdown_content,
    })
}

// ---------------------------------------------------------------------------
// napi-rs wrappers (JavaScript interop)
// ---------------------------------------------------------------------------

/// Non-generic result struct for napi-rs.
///
/// The core [`ParseResult<T>`] is generic over `T: DeserializeOwned`, which
/// napi-rs does not support.  This mirror struct deserialises the YAML
/// front-matter into a `serde_json::Value` (i.e. a plain JS object) instead.
#[cfg(feature = "napi")]
#[cfg_attr(feature = "napi", napi(object))]
pub struct ParseResultJs {
    /// The Markdown body with the YAML front matter removed.
    pub pure_markdown_content: String,
    /// The deserialised YAML front-matter as a JSON string (or `None`).
    ///
    /// A JSON string is used because napi-rs cannot directly return
    /// `serde_json::Value`; the JS side can `JSON.parse()` it.
    pub yaml_data: Option<String>,
}

/// napi-exported wrapper of [`parse_markdown_frontmatter`].
///
/// Deserialises the YAML front-matter into a `serde_json::Value` and
/// returns the result as a [`ParseResultJs`] where `yaml_data` is a
/// JSON string (or `None` if no front-matter was present).
#[cfg(feature = "napi")]
#[cfg_attr(feature = "napi", napi)]
pub fn parse_markdown_frontmatter_js(content: String) -> Result<ParseResultJs, napi::Error> {
    let result = parse_markdown_frontmatter::<serde_json::Value>(&content)
        .map_err(|e| napi::Error::from_reason(e))?;
    let yaml_data = result
        .yaml_data
        .map(|v| serde_json::to_string(&v).unwrap_or_default());
    Ok(ParseResultJs {
        pure_markdown_content: result.pure_markdown_content,
        yaml_data,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_markdown_frontmatter;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Frontmatter {
        title: String,
        author: String,
    }

    #[test]
    fn parses_frontmatter_and_strips_it_from_markdown() {
        let content = "---\ntitle: Hello\nauthor: world\n---\n# Heading\n\nThis is body text.\n";

        let result = parse_markdown_frontmatter::<Frontmatter>(content).unwrap();

        assert_eq!(
            result.yaml_data,
            Some(Frontmatter {
                title: "Hello".to_string(),
                author: "world".to_string(),
            })
        );
        assert_eq!(
            result.pure_markdown_content,
            "# Heading\n\nThis is body text.\n"
        );
    }

    #[test]
    fn keeps_markdown_when_frontmatter_is_absent() {
        let content = "# Heading\n\nThis is body text.\n";

        let result = parse_markdown_frontmatter::<Frontmatter>(content).unwrap();

        assert_eq!(result.yaml_data, None);
        assert_eq!(result.pure_markdown_content, content);
    }

    #[test]
    fn accepts_leading_whitespace_and_crlf_frontmatter() {
        let content = "\r\n---\r\ntitle: Hello\r\nauthor: world\r\n---\r\n\r\nBody text\n";

        let result = parse_markdown_frontmatter::<Frontmatter>(content).unwrap();

        assert_eq!(
            result.yaml_data,
            Some(Frontmatter {
                title: "Hello".to_string(),
                author: "world".to_string(),
            })
        );
        assert_eq!(result.pure_markdown_content, "Body text\n");
    }
}
