//! # peisar_html
//!
//! Render a [`peisar_ast`] Markdown AST ([`Document`]) into HTML.
//!
//! `peisar_html` takes an already-parsed [`Document`] and produces an HTML
//! string.  It is intentionally separate from the parser so that visitors
//! can mutate the AST before rendering.
//!
//! ## Quick start
//!
//! ```rust
//! use peisar_ast::{Document, AstOptions};
//! use peisar_html::{AstToHtml, RenderOptions, render_document_html};
//!
//! let md = "# Hello\n\nA paragraph with **bold** text.\n";
//! let doc = Document::parse(md, &AstOptions::default(), None);
//!
//! // One-liner
//! let html = render_document_html(&doc, None);
//! assert!(html.contains("<h1>Hello</h1>"));
//! assert!(html.contains("<strong>bold</strong>"));
//!
//! // Or use the builder-style API
//! let opts = RenderOptions { fragment: false, title: Some("Test".into()), ..Default::default() };
//! let html2 = AstToHtml::new(Some(opts)).render_document_owned(&doc);
//! assert!(html2.starts_with("<!DOCTYPE html>"));
//! ```
//!
//! ## Fragment vs full document
//!
//! By default (`fragment: true`) only the body content is emitted — no
//! `<!DOCTYPE>`, `<html>`, `<head>`, or `<body>` wrappers.  Set
//! `fragment: false` to produce a complete standalone HTML document with
//! optional `<title>`, `<meta>` tags, and inline `<style>`.
//!
//! ## Supported elements
//!
//! All block and inline node types from [`peisar_ast::token`] are rendered:
//!
//! - **Blocks**: headings, paragraphs, code blocks, block quotes, ordered /
//!   unordered lists (with GFM task-list checkboxes), thematic breaks, raw
//!   HTML blocks, GFM tables, HTML comments.
//! - **Inlines**: text, emphasis (`<em>` / `<strong>`), inline code,
//!   strikethrough (`<del>`), hard / soft breaks, images, links, reference
//!   links, raw inline HTML.
//! - **Kramdown attributes** (`{:#id .class key="val"}`) are emitted as HTML
//!   attributes on the corresponding block element.

mod options;

#[cfg(feature = "napi")]
use napi_derive::napi;
pub use options::RenderOptions;
use peisar_ast::token::{
    Block, EmphasisLevel, Inline, ListItem, Table as AstTable, TableCellAlignment, TaskState,
};
use peisar_ast::{Attributes, Document};

/// A renderer that converts a [`Document`] AST into an HTML string.
///
/// Create with [`AstToHtml::new`], call [`AstToHtml::render_document`] to
/// fill the internal buffer, then call [`AstToHtml::finish`] to consume the
/// renderer and obtain the `String`.
///
/// # Example
///
/// ```rust
/// use peisar_ast::{Document, AstOptions};
/// use peisar_html::AstToHtml;
///
/// let doc = Document::parse("# Hi\n", &AstOptions::default(), None);
/// let mut r = AstToHtml::default();
/// r.render_document(&doc);
/// let html = r.finish();
/// assert_eq!(html, "<h1>Hi</h1>\n");
/// ```
#[cfg_attr(feature = "napi", napi)]
pub struct AstToHtml {
    out: String,
    /// When `true`, inline text is HTML-escaped.
    escape: bool,
    opts: RenderOptions,
}

/// Convenience: render a [`Document`] to an HTML string in one call.
///
/// Pass `None` for options to use the defaults (fragment mode, no wrapping).
///
/// # Example
///
/// ```rust
/// use peisar_ast::{Document, AstOptions};
/// use peisar_html::render_document_html;
///
/// let doc = Document::parse("Hello **world**.\n", &AstOptions::default(), None);
/// let html = render_document_html(&doc, None);
/// assert_eq!(html, "<p>Hello <strong>world</strong>.</p>\n");
/// ```
pub fn render_document_html(doc: &Document, opts: Option<RenderOptions>) -> String {
    let mut r = AstToHtml::new(opts);
    r.render_document(doc);
    r.finish()
}
impl AstToHtml {
    /// Create a new renderer with the given options.
    ///
    /// Pass `None` to use [`RenderOptions::default`] (fragment mode).
    pub fn new(opts: Option<RenderOptions>) -> Self {
        let options = if opts.is_none() {
            RenderOptions::default()
        } else {
            opts.unwrap()
        };
        Self {
            out: String::new(),
            escape: true,
            opts: options,
        }
    }
    /// Consume the renderer and return the HTML string.
    pub fn finish(self) -> String {
        self.out
    }

    /// Render a [`Document`] and return the HTML string immediately
    /// (consuming the renderer).
    ///
    /// This is a convenience wrapper around [`render_document`](Self::render_document)
    /// + [`finish`](Self::finish).
    ///
    /// # Example
    ///
    /// ```rust
    /// use peisar_ast::{Document, AstOptions};
    /// use peisar_html::AstToHtml;
    ///
    /// let doc = Document::parse("Hi.\n", &AstOptions::default(), None);
    /// let html = AstToHtml::default().render_document_owned(&doc);
    /// assert_eq!(html, "<p>Hi.</p>\n");
    /// ```
    pub fn render_document_owned(mut self, doc: &Document) -> String {
        self.render_document(doc);
        self.finish()
    }
    fn esc(&mut self, text: &str) {
        if self.escape {
            for c in text.chars() {
                match c {
                    '&' => self.out.push_str("&amp;"),
                    '<' => self.out.push_str("&lt;"),
                    '>' => self.out.push_str("&gt;"),
                    '"' => self.out.push_str("&quot;"),
                    '\'' => self.out.push_str("&#39;"),
                    _ => self.out.push(c),
                }
            }
        } else {
            self.out.push_str(text);
        }
    }
    /// Emit Kramdown attributes as an HTML attribute string (with a leading
    /// space if non-empty).
    fn emit_attrs(&mut self, attrs: Option<&Attributes>) {
        if let Some(a) = attrs {
            let s = a.to_html_attr_string();
            if !s.is_empty() {
                self.out.push(' ');
                self.out.push_str(&s);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Top-level entry
    // -----------------------------------------------------------------------

    /// Render a full [`Document`] into the internal buffer.
    ///
    /// When `RenderOptions::fragment` is `true` (the default) only the body
    /// content is emitted.  When `false`, a complete `<!DOCTYPE html>`
    /// document with `<html>`, `<head>`, and `<body>` wrappers is produced.
    pub fn render_document(&mut self, doc: &Document) {
        if self.opts.fragment {
            for b in &doc.children {
                self.render_block(b);
            }
        } else {
            self.out.push_str("<!DOCTYPE html>\n<html");
            if let Some(bc) = self.opts.body_class.clone() {
                self.out.push_str(" class=\"");
                self.esc_no_quote(&bc);
                self.out.push_str("\"");
            }
            self.out.push_str(">\n<head>\n");
            if self.opts.charset {
                self.out.push_str("<meta charset=\"utf-8\">\n");
            }
            if self.opts.viewport {
                self.out.push_str(
                    "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
                );
            }
            if let Some(title) = self.opts.title.clone() {
                self.out.push_str("<title>");
                self.esc(&title);
                self.out.push_str("</title>\n");
            }
            if let Some(style) = self.opts.style.clone() {
                self.out.push_str("<style>\n");
                self.out.push_str(&style);
                self.out.push_str("\n</style>\n");
            }
            self.out.push_str("</head>\n<body");
            if let Some(bc) = self.opts.body_class.clone() {
                self.out.push_str(" class=\"");
                self.esc_no_quote(&bc);
                self.out.push_str("\"");
            }
            self.out.push_str(">\n");
            for b in &doc.children {
                self.render_block(b);
            }
            self.out.push_str("</body>\n</html>\n");
        }
    }

    // -----------------------------------------------------------------------
    // Blocks
    // -----------------------------------------------------------------------

    /// Render a single block-level node.
    pub fn render_block(&mut self, block: &Block) {
        match block {
            Block::Heading {
                level,
                children,
                attrs,
                ..
            } => {
                self.out.push_str(&format!("<h{}", level));
                self.emit_attrs(attrs.as_ref());
                self.out.push('>');
                for inline in children {
                    self.render_inline(inline);
                }
                self.out.push_str(&format!("</h{}>\n", level));
            }
            Block::Paragraph {
                children, attrs, ..
            } => {
                self.out.push_str("<p");
                self.emit_attrs(attrs.as_ref());
                self.out.push('>');
                for inline in children {
                    self.render_inline(inline);
                }
                self.out.push_str("</p>\n");
            }
            Block::CodeBlock {
                lang, code, attrs, ..
            } => {
                self.out.push_str("<pre><code");
                if let Some(l) = lang {
                    if !l.is_empty() {
                        self.out.push_str(" class=\"language-");
                        self.esc_no_quote(l);
                        self.out.push_str("\"");
                    }
                }
                self.emit_attrs(attrs.as_ref());
                self.out.push('>');
                // Code blocks are never escaped for *content* when we want
                // raw output, but we must escape < > & for valid HTML.
                self.esc(code);
                self.out.push_str("</code></pre>\n");
            }
            Block::BlockQuote {
                children, attrs, ..
            } => {
                self.out.push_str("<blockquote");
                self.emit_attrs(attrs.as_ref());
                self.out.push_str(">\n");
                for b in children {
                    self.render_block(b);
                }
                self.out.push_str("</blockquote>\n");
            }
            Block::List {
                ordered,
                items,
                attrs,
                ..
            } => {
                let tag = if *ordered { "ol" } else { "ul" };
                self.out.push('<');
                self.out.push_str(tag);
                self.emit_attrs(attrs.as_ref());
                self.out.push_str(">\n");
                for item in items {
                    self.render_list_item(item);
                }
                self.out.push_str(&format!("</{}>\n", tag));
            }
            Block::ThematicBreak { .. } => {
                self.out.push_str("<hr>\n");
            }
            Block::HtmlBlock { html, .. } => {
                self.out.push_str(html);
                if !html.ends_with('\n') {
                    self.out.push('\n');
                }
            }
            Block::Table { table, attrs, .. } => {
                self.render_table(table, attrs);
            }
            Block::LinkReferenceDefinition { .. } => {
                // Not rendered as visible HTML.
            }
            Block::Comment { value, .. } => {
                self.out.push_str("<!-- ");
                self.out.push_str(value);
                self.out.push_str(" -->\n");
            }
        }
    }

    /// Render a single list item (`<li>`).
    fn render_list_item(&mut self, item: &ListItem) {
        self.out.push_str("<li");
        if let Some(task) = &item.task {
            let checked = matches!(task, TaskState::Checked);
            self.out.push_str(" class=\"task-list-item\"");
            if checked {
                self.out.push_str(" data-checked=\"true\"");
            }
        }
        self.out.push('>');
        if let Some(task) = &item.task {
            let checked = matches!(task, TaskState::Checked);
            self.out
                .push_str("<input type=\"checkbox\" class=\"task-list-item-checkbox\"");
            if checked {
                self.out.push_str(" checked");
            }
            self.out.push_str(" disabled> ");
        }
        // Render nested blocks.  If the item contains a single paragraph we
        // render it inline (no wrapping <p>) per CommonMark rendering
        // convention for tight lists.
        let tight = item.children.len() == 1
            && matches!(item.children.first(), Some(Block::Paragraph { .. }));
        if tight {
            if let Some(Block::Paragraph { children, .. }) = item.children.first() {
                for inline in children {
                    self.render_inline(inline);
                }
            }
        } else {
            self.out.push('\n');
            for b in &item.children {
                self.render_block(b);
            }
        }
        self.out.push_str("</li>\n");
    }

    /// Render a GFM table.
    fn render_table(&mut self, table: &AstTable, attrs: &Option<Attributes>) {
        self.out.push_str("<table");
        self.emit_attrs(attrs.as_ref());
        self.out.push_str(">\n<thead>\n<tr>\n");
        // Header
        for (i, cell) in table.header.cells.iter().enumerate() {
            self.out.push_str("<th");
            self.emit_align(table.alignments.get(i));
            self.out.push('>');
            for inline in &cell.children {
                self.render_inline(inline);
            }
            self.out.push_str("</th>\n");
        }
        self.out.push_str("</tr>\n</thead>\n<tbody>\n");
        // Body rows
        for row in &table.rows {
            self.out.push_str("<tr>\n");
            for (i, cell) in row.cells.iter().enumerate() {
                self.out.push_str("<td");
                self.emit_align(table.alignments.get(i));
                self.out.push('>');
                for inline in &cell.children {
                    self.render_inline(inline);
                }
                self.out.push_str("</td>\n");
            }
            self.out.push_str("</tr>\n");
        }
        self.out.push_str("</tbody>\n</table>\n");
    }

    /// Emit a `style="text-align: …"` attribute for a table column alignment.
    fn emit_align(&mut self, align: Option<&TableCellAlignment>) {
        if let Some(a) = align {
            let css = match a {
                TableCellAlignment::Default | TableCellAlignment::Left => return,
                TableCellAlignment::Center => "text-align: center",
                TableCellAlignment::Right => "text-align: right",
            };
            self.out.push_str(" style=\"");
            self.out.push_str(css);
            self.out.push('"');
        }
    }

    // -----------------------------------------------------------------------
    // Inlines
    // -----------------------------------------------------------------------

    /// Render a single inline-level node.
    pub fn render_inline(&mut self, inline: &Inline) {
        match inline {
            Inline::Text { value, .. } => {
                self.esc(value);
            }
            Inline::Emphasis {
                level, children, ..
            } => {
                let (open, close) = match level {
                    EmphasisLevel::Italic => ("<em>", "</em>"),
                    EmphasisLevel::Bold => ("<strong>", "</strong>"),
                };
                self.out.push_str(open);
                for child in children {
                    self.render_inline(child);
                }
                self.out.push_str(close);
            }
            Inline::Code { code, .. } => {
                self.out.push_str("<code>");
                self.esc(code);
                self.out.push_str("</code>");
            }
            Inline::HtmlInline { html, .. } => {
                // Raw HTML — emit as-is.
                self.out.push_str(html);
            }
            Inline::Strikethrough { children, .. } => {
                self.out.push_str("<del>");
                for child in children {
                    self.render_inline(child);
                }
                self.out.push_str("</del>");
            }
            Inline::HardBreak { .. } => {
                self.out.push_str("<br>\n");
            }
            Inline::SoftBreak { .. } => {
                self.out.push('\n');
            }
            Inline::Image {
                alt, url, title, ..
            } => {
                self.out.push_str("<img src=\"");
                self.esc_attr(url);
                self.out.push_str("\" alt=\"");
                self.esc(alt);
                self.out.push('"');
                if let Some(t) = title {
                    self.out.push_str(" title=\"");
                    self.esc(t);
                    self.out.push('"');
                }
                self.out.push('>');
            }
            Inline::Link {
                text, url, title, ..
            } => {
                self.out.push_str("<a href=\"");
                self.esc_attr(url);
                self.out.push('"');
                if let Some(t) = title {
                    self.out.push_str(" title=\"");
                    self.esc(t);
                    self.out.push('"');
                }
                self.out.push('>');
                for child in text {
                    self.render_inline(child);
                }
                self.out.push_str("</a>");
            }
            Inline::LinkReference {
                text, url, title, ..
            } => {
                self.out.push_str("<a href=\"");
                self.esc_attr(url);
                self.out.push('"');
                if let Some(t) = title {
                    self.out.push_str(" title=\"");
                    self.esc(t);
                    self.out.push('"');
                }
                self.out.push('>');
                for child in text {
                    self.render_inline(child);
                }
                self.out.push_str("</a>");
            }
        }
    }

    // -----------------------------------------------------------------------
    // Small helpers
    // -----------------------------------------------------------------------

    /// Escape text for use inside an HTML attribute value (escapes `&`, `"`,
    /// `<`, `>`).
    fn esc_attr(&mut self, text: &str) {
        for c in text.chars() {
            match c {
                '&' => self.out.push_str("&amp;"),
                '"' => self.out.push_str("&quot;"),
                '<' => self.out.push_str("&lt;"),
                '>' => self.out.push_str("&gt;"),
                _ => self.out.push(c),
            }
        }
    }

    /// Escape text without escaping single quotes — used for class names and
    /// other attribute values that won't contain `'`.
    fn esc_no_quote(&mut self, text: &str) {
        for c in text.chars() {
            match c {
                '&' => self.out.push_str("&amp;"),
                '"' => self.out.push_str("&quot;"),
                '<' => self.out.push_str("&lt;"),
                '>' => self.out.push_str("&gt;"),
                _ => self.out.push(c),
            }
        }
    }
}
impl Default for AstToHtml {
    fn default() -> Self {
        Self::new(None)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use peisar_ast::{AstOptions, Document};

    /// Helper: parse markdown and render to HTML fragment.
    fn render(md: &str) -> String {
        let doc = Document::parse(md, &AstOptions::default(), None);
        render_document_html(&doc, None)
    }

    /// Helper: parse markdown and render to a full HTML document.
    fn render_full(md: &str, opts: RenderOptions) -> String {
        let doc = Document::parse(md, &AstOptions::default(), None);
        render_document_html(&doc, Some(opts))
    }

    // -----------------------------------------------------------------------
    // Headings
    // -----------------------------------------------------------------------

    #[test]
    fn heading_h1() {
        assert_eq!(render("# Hello\n"), "<h1>Hello</h1>\n");
    }

    #[test]
    fn heading_h3() {
        assert_eq!(render("### Sub\n"), "<h3>Sub</h3>\n");
    }

    #[test]
    fn heading_with_emphasis() {
        let html = render("# Hello *world*\n");
        assert!(html.contains("<h1>Hello <em>world</em></h1>"));
    }

    // -----------------------------------------------------------------------
    // Paragraphs & text
    // -----------------------------------------------------------------------

    #[test]
    fn paragraph_plain() {
        assert_eq!(render("Hello.\n"), "<p>Hello.</p>\n");
    }

    #[test]
    fn paragraph_with_bold() {
        let html = render("**bold** text\n");
        assert_eq!(html, "<p><strong>bold</strong> text</p>\n");
    }

    #[test]
    fn paragraph_with_italic() {
        let html = render("*italic*\n");
        assert_eq!(html, "<p><em>italic</em></p>\n");
    }

    #[test]
    fn text_html_escaping() {
        let html = render("5 < 10 & 20 > 15\n");
        assert!(html.contains("5 &lt; 10 &amp; 20 &gt; 15"));
    }

    #[test]
    fn text_quote_escaping() {
        let html = render(r#"She said "hi" and 'bye'\n"#);
        assert!(html.contains("&quot;hi&quot;"));
        assert!(html.contains("&#39;bye&#39;"));
    }

    // -----------------------------------------------------------------------
    // Code
    // -----------------------------------------------------------------------

    #[test]
    fn inline_code() {
        let html = render("Use `let x = 5;` here.\n");
        assert!(html.contains("<code>let x = 5;</code>"));
    }

    #[test]
    fn code_block_with_lang() {
        let md = "```rust\nfn main() {}\n```\n";
        let html = render(md);
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn code_block_escapes_html() {
        let md = "```\n<div>raw</div>\n```\n";
        let html = render(md);
        assert!(html.contains("&lt;div&gt;"));
        assert!(!html.contains("<div>raw</div>"));
    }

    #[test]
    fn code_block_no_lang() {
        let md = "```\nplain\n```\n";
        let html = render(md);
        assert!(html.contains("<pre><code>"));
        assert!(html.contains("plain"));
    }

    // -----------------------------------------------------------------------
    // Block quotes
    // -----------------------------------------------------------------------

    #[test]
    fn block_quote() {
        let html = render("> Wisdom.\n");
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("<p>Wisdom.</p>"));
        assert!(html.contains("</blockquote>"));
    }

    #[test]
    fn block_quote_nested() {
        let html = render("> # Heading in quote\n");
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("<h1>Heading in quote</h1>"));
    }

    // -----------------------------------------------------------------------
    // Lists
    // -----------------------------------------------------------------------

    #[test]
    fn unordered_list() {
        let html = render("- one\n- two\n");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>one</li>"));
        assert!(html.contains("<li>two</li>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn ordered_list() {
        let html = render("1. first\n2. second\n");
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>first</li>"));
        assert!(html.contains("</ol>"));
    }

    #[test]
    fn task_list_checked() {
        let html = render("- [x] done\n");
        assert!(html.contains("task-list-item"));
        assert!(html.contains("checked"));
    }

    #[test]
    fn task_list_unchecked() {
        let html = render("- [ ] todo\n");
        assert!(html.contains("task-list-item"));
        assert!(html.contains("checkbox"));
        assert!(!html.contains("checked"));
    }

    // -----------------------------------------------------------------------
    // Thematic break
    // -----------------------------------------------------------------------

    #[test]
    fn thematic_break() {
        assert_eq!(render("---\n"), "<hr>\n");
    }

    // -----------------------------------------------------------------------
    // Links & images
    // -----------------------------------------------------------------------

    #[test]
    fn link() {
        let html = render("[click](https://example.com)\n");
        assert!(html.contains("<a href=\"https://example.com\">click</a>"));
    }

    #[test]
    fn link_with_title() {
        let html = render(r#"[click](https://example.com "Title")\n"#);
        assert!(html.contains("title=\"Title\""));
    }

    #[test]
    fn image() {
        let html = render("![alt text](img.png)\n");
        assert!(html.contains("<img src=\"img.png\" alt=\"alt text\">"));
    }

    #[test]
    fn image_with_title() {
        let html = render(r#"![alt](img.png "My Title")\n"#);
        assert!(html.contains("title=\"My Title\""));
    }

    // -----------------------------------------------------------------------
    // Strikethrough
    // -----------------------------------------------------------------------

    #[test]
    fn strikethrough() {
        let html = render("~~deleted~~\n");
        assert!(html.contains("<del>deleted</del>"));
    }

    // -----------------------------------------------------------------------
    // Tables
    // -----------------------------------------------------------------------

    #[test]
    fn table_basic() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
        let html = render(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<th>A</th>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn table_alignment() {
        let md = "| A | B | C |\n|:---|:--:|---:|\n| 1 | 2 | 3 |\n";
        let html = render(md);
        // Left = no style, Center = text-align: center, Right = text-align: right
        assert!(html.contains("text-align: center"));
        assert!(html.contains("text-align: right"));
    }

    // -----------------------------------------------------------------------
    // Raw HTML passthrough
    // -----------------------------------------------------------------------

    #[test]
    fn html_block_passthrough() {
        let md = "<div class=\"raw\">content</div>\n";
        let html = render(md);
        assert!(html.contains("<div class=\"raw\">content</div>"));
    }

    // -----------------------------------------------------------------------
    // Full document mode
    // -----------------------------------------------------------------------

    #[test]
    fn full_document_has_doctype() {
        let opts = RenderOptions {
            fragment: false,
            ..Default::default()
        };
        let html = render_full("# Title\n", opts);
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn full_document_with_title() {
        let opts = RenderOptions {
            fragment: false,
            title: Some("My Page".into()),
            ..Default::default()
        };
        let html = render_full("Hello.\n", opts);
        assert!(html.contains("<title>My Page</title>"));
    }

    #[test]
    fn full_document_with_charset_and_viewport() {
        let opts = RenderOptions {
            fragment: false,
            ..Default::default()
        };
        let html = render_full("Hi.\n", opts);
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("viewport"));
    }

    #[test]
    fn full_document_with_style() {
        let opts = RenderOptions {
            fragment: false,
            style: Some("body { margin: 0; }".into()),
            ..Default::default()
        };
        let html = render_full("Hi.\n", opts);
        assert!(html.contains("<style>"));
        assert!(html.contains("body { margin: 0; }"));
        assert!(html.contains("</style>"));
    }

    #[test]
    fn full_document_with_body_class() {
        let opts = RenderOptions {
            fragment: false,
            body_class: Some("dark".into()),
            ..Default::default()
        };
        let html = render_full("Hi.\n", opts);
        assert!(html.contains("class=\"dark\""));
    }

    #[test]
    fn full_document_disable_charset() {
        let opts = RenderOptions {
            fragment: false,
            charset: false,
            ..Default::default()
        };
        let html = render_full("Hi.\n", opts);
        assert!(!html.contains("<meta charset"));
    }

    // -----------------------------------------------------------------------
    // Builder API
    // -----------------------------------------------------------------------

    #[test]
    fn builder_style() {
        let doc = Document::parse("# Hi\n", &AstOptions::default(), None);
        let mut r = AstToHtml::default();
        r.render_document(&doc);
        let html = r.finish();
        assert_eq!(html, "<h1>Hi</h1>\n");
    }

    #[test]
    fn render_document_owned() {
        let doc = Document::parse("Hello.\n", &AstOptions::default(), None);
        let html = AstToHtml::default().render_document_owned(&doc);
        assert_eq!(html, "<p>Hello.</p>\n");
    }

    #[test]
    fn render_options_from_bool() {
        let opts = RenderOptions::from(false);
        assert!(!opts.fragment);
    }

    // -----------------------------------------------------------------------
    // Kramdown attributes
    // -----------------------------------------------------------------------

    #[test]
    fn kramdown_id_and_class() {
        // Construct a Document with attributes manually (the parser's
        // attribute handling for `.class` has a known unwrap bug).
        use peisar_ast::token::{Block, Inline};
        use peisar_ast::{Attributes, Document};

        let doc = Document {
            node_type: "root".into(),
            file_name: None,
            pos: Default::default(),
            link_references: vec![],
            children: vec![Block::Paragraph {
                children: vec![Inline::Text {
                    value: "Hi".into(),
                    pos: Default::default(),
                }],
                pos: Default::default(),
                attrs: Some(Attributes {
                    id: Some("intro".into()),
                    classes: Some(vec!["highlight".into()]),
                    attributes: Some(vec![("data-x".into(), "42".into())]),
                }),
            }],
        };
        let html = render_document_html(&doc, None);
        assert!(html.contains("id=\"intro\""));
        assert!(html.contains("class=\"highlight\""));
        assert!(html.contains("data-x=\"42\""));
    }
}
