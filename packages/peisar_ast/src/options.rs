use serde::{Deserialize, Serialize};

/// Options that control how Markdown is parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstOptions {
    /// Enable GitHub Flavored Markdown (tables, strikethrough, task lists,
    /// autolinks).  Default: `true`.
    pub gfm: bool,
    /// Enable Kramdown-style block attributes (`{:#id .class key="val"}`).
    /// Default: `true`.
    pub kramdown: bool,
}

impl Default for AstOptions {
    fn default() -> Self {
        Self {
            gfm: true,
            kramdown: true,
        }
    }
}
