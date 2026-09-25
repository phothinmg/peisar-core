#[cfg(feature = "napi")]
use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Options that control how the AST is rendered to HTML.
#[cfg_attr(feature = "napi", napi(object))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderOptions {
    /// If `true`, emit only the body content (no `<!DOCTYPE>`, `<html>`,
    /// `<head>`, or `<body>` wrapper).  If `false`, emit a full HTML
    /// document.  Default: `true` (fragment).
    pub fragment: bool,
    /// Include a `<meta charset="utf-8">` in the head (only relevant when
    /// `fragment` is `false`).  Default: `true`.
    pub charset: bool,
    /// Include a `<meta name="viewport" content="width=device-width,
    /// initial-scale=1.0">` in the head (only relevant when `fragment` is
    /// `false`).  Default: `true`.
    pub viewport: bool,
    /// Optional `<title>` for the HTML head (only relevant when
    /// `fragment` is `false`).  Default: `None`.
    pub title: Option<String>,
    /// Optional additional CSS classes to add to `<body>` (only relevant
    /// when `fragment` is `false`).  Default: `None`.
    pub body_class: Option<String>,
    /// Optional inline CSS to inject in a `<style>` tag in the head.
    /// Default: `None`.
    pub style: Option<String>,
}
impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            fragment: true,
            charset: true,
            viewport: true,
            title: None,
            body_class: None,
            style: None,
        }
    }
}

/// Convenience: create `Some(true)` for fragment mode (backward-compatible
/// with the old `render_document(doc, Some(bool))` API).
impl From<bool> for RenderOptions {
    fn from(fragment: bool) -> Self {
        Self {
            fragment,
            ..Default::default()
        }
    }
}
