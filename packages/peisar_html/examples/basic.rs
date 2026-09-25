//! Basic example: parse Markdown to AST and render to HTML.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p peisar_html --example basic
//! ```

use peisar_ast::{AstOptions, Document};
use peisar_html::{AstToHtml, RenderOptions, render_document_html};

const SAMPLE_MD: &str = r#"# peisar_html demo

A paragraph with **bold**, *italic*, ~~strikethrough~~, and `inline code`.

## Lists

- Unordered item 1
- [x] Completed task
- [ ] Pending task

1. Ordered first
2. Ordered second

## Links and images

A [link](https://example.com "Example") and an image:

![logo](https://example.com/logo.png "Logo")

## Code block

```rust
fn main() {
    println!("Hello from peisar_html!");
}
```

## Table

| Feature    | Supported |
|:-----------|:---------:|
| Tables     | ✅        |
| Task lists | ✅        |

> A block quote with *emphasis*.

---

<!-- This is an HTML comment -->

{:#custom-id .custom-class}
A paragraph with Kramdown attributes.
"#;

fn main() {
    // ── Fragment mode (default) ──────────────────────────────────────────
    let doc = Document::parse(SAMPLE_MD, &AstOptions::default(), None);

    println!("══════════════════════════════════════════════════════");
    println!("  Fragment mode (body content only)");
    println!("══════════════════════════════════════════════════════\n");

    let fragment_html = render_document_html(&doc, None);
    println!("{fragment_html}");

    // ── Full document mode ───────────────────────────────────────────────
    let doc2 = Document::parse(SAMPLE_MD, &AstOptions::default(), None);

    println!("\n══════════════════════════════════════════════════════");
    println!("  Full document mode (with <head>)");
    println!("══════════════════════════════════════════════════════\n");

    let opts = RenderOptions {
        fragment: false,
        title: Some("peisar_html Demo".into()),
        body_class: Some("markdown-body".into()),
        style: Some("body { font-family: sans-serif; max-width: 720px; margin: 2em auto; }".into()),
        ..Default::default()
    };

    let full_html = AstToHtml::new(Some(opts)).render_document_owned(&doc2);
    println!("{full_html}");
}
