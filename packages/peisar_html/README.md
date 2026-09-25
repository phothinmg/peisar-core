<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->
<div align="center">
<img src="https://pub-c9ba018358dd48a99b70013b65a25e5f.r2.dev/logo/peisar.webp" width="160" height="160" alt="peisar" />
  <h1>Peisar</h1>
  <p>Peisar HTML</p>
</div>

> Render a [`peisar_ast`] Markdown AST into HTML.

[![crates.io](https://img.shields.io/crates/v/peisar_html.svg)](https://crates.io/crates/peisar_html)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`peisar_html` takes an already-parsed [`Document`][doc] from the
[`peisar_ast`][ast] crate and produces a well-formed HTML string. It is
intentionally decoupled from the parser so that [visitors][visitor] can
mutate the AST before rendering.

---

## Features

- **Full element coverage** — every block and inline node type from
  `peisar_ast::token` is rendered:
  - **Blocks**: headings, paragraphs, fenced / indented code blocks, block
    quotes, ordered & unordered lists, GFM task-list checkboxes, thematic
    breaks, raw HTML blocks, GFM tables, HTML comments.
  - **Inlines**: text, emphasis (`<em>` / `<strong>`), inline code,
    strikethrough (`<del>`), hard & soft line breaks, images, links,
    reference links, raw inline HTML.
- **Kramdown attributes** — `{:#id .class key="val"}` are emitted as real
  HTML attributes on the corresponding element.
- **GFM table alignment** — `:---`, `:--:`, `---:` produce
  `style="text-align: …"` on `<th>` / `<td>`.
- **Fragment or full document** — emit just the body content (default) or a
  complete `<!DOCTYPE html>` document with `<head>`, `<title>`, `<meta>`,
  and optional inline `<style>`.
- **HTML escaping** — text content and attribute values are escaped by
  default (`&`, `<`, `>`, `"`, `'`).

---

## Quick start

### As a library

```toml
[dependencies]
peisar_ast  = "0.1"
peisar_html = "0.1"
```

```rust
use peisar_ast::{Document, AstOptions};
use peisar_html::render_document_html;

let md = "# Hello\n\nA paragraph with **bold** and *italic*.\n";
let doc = Document::parse(md, &AstOptions::default(), None);

let html = render_document_html(&doc, None);
println!("{html}");
```

Output:

```html
<h1>Hello</h1>
<p>A paragraph with <strong>bold</strong> and <em>italic</em>.</p>
```

### Full document mode

```rust
use peisar_html::RenderOptions;

let opts = RenderOptions {
    fragment: false,                       // produce a full <html> document
    title: Some("My Page".into()),
    body_class: Some("markdown-body".into()),
    style: Some("body { max-width: 720px; }".into()),
    ..Default::default()
};

let html = render_document_html(&doc, Some(opts));
assert!(html.starts_with("<!DOCTYPE html>"));
```

### Builder API

```rust
use peisar_html::AstToHtml;

let mut renderer = AstToHtml::default();
renderer.render_document(&doc);
let html = renderer.finish();
```

Or in one shot:

```rust
let html = AstToHtml::default().render_document_owned(&doc);
```

---

## Examples

Run the bundled example:

```sh
cargo run -p peisar_html --example basic
```

This demonstrates both fragment mode and full-document mode with a
comprehensive Markdown sample (headings, lists, task lists, tables, code
blocks, links, images, block quotes, Kramdown attributes, and more).

---

## API reference

### `render_document_html`

```rust
pub fn render_document_html(doc: &Document, opts: Option<RenderOptions>) -> String
```

One-shot convenience function. Pass `None` for default options (fragment
mode).

### `AstToHtml`

| Method                        | Description                                                    |
| ----------------------------- | -------------------------------------------------------------- |
| `new(opts)`                   | Create a renderer with the given options (`None` = defaults).  |
| `render_document(&doc)`       | Render a `Document` into the internal buffer.                  |
| `render_document_owned(&doc)` | Render + consume, returning `String`.                          |
| `finish()`                    | Consume the renderer, returning the accumulated HTML `String`. |
| `render_block(&block)`        | Render a single `Block` node.                                  |
| `render_inline(&inline)`      | Render a single `Inline` node.                                 |

### `RenderOptions`

| Field        | Type             | Default | Description                                                    |
| ------------ | ---------------- | ------- | -------------------------------------------------------------- |
| `fragment`   | `bool`           | `true`  | `true` = body only; `false` = full `<!DOCTYPE html>` document. |
| `charset`    | `bool`           | `true`  | Emit `<meta charset="utf-8">` (full-doc only).                 |
| `viewport`   | `bool`           | `true`  | Emit viewport `<meta>` (full-doc only).                        |
| `title`      | `Option<String>` | `None`  | `<title>` text (full-doc only).                                |
| `body_class` | `Option<String>` | `None`  | CSS class for `<body>` (full-doc only).                        |
| `style`      | `Option<String>` | `None`  | Inline CSS for `<style>` in `<head>`.                          |

`RenderOptions` also implements `From<bool>` for backward compatibility
(`RenderOptions::from(false)` → full-document mode).

Generate full API docs with:

```sh
cargo doc -p peisar_html --open
```

---

## Rendering reference

### Block elements

| Markdown                | HTML                                                      |
| ----------------------- | --------------------------------------------------------- |
| `# H1` … `###### H6`    | `<h1>` … `<h6>`                                           |
| Plain paragraph         | `<p>…</p>`                                                |
| ` ```lang ` fenced code | `<pre><code class="language-lang">…</code></pre>`         |
| `> quote`               | `<blockquote>…</blockquote>`                              |
| `- item` / `1. item`    | `<ul>` / `<ol>` with `<li>`                               |
| `- [x] task`            | `<li class="task-list-item"><input … checked disabled> …` |
| `---`                   | `<hr>`                                                    |
| GFM table               | `<table><thead>…</thead><tbody>…</tbody></table>`         |
| Raw HTML block          | Passthrough                                               |
| `<!-- comment -->`      | `<!-- comment -->`                                        |

### Inline elements

| Markdown        | HTML                        |
| --------------- | --------------------------- |
| `**bold**`      | `<strong>bold</strong>`     |
| `*italic*`      | `<em>italic</em>`           |
| `` `code` ``    | `<code>code</code>`         |
| `~~strike~~`    | `<del>strike</del>`         |
| `[text](url)`   | `<a href="url">text</a>`    |
| `![alt](url)`   | `<img src="url" alt="alt">` |
| Hard break      | `<br>`                      |
| Soft break      | `\n`                        |
| Raw inline HTML | Passthrough                 |

---

## License

Apache-2.0

[ast]: https://crates.io/crates/peisar_ast
[doc]: https://docs.rs/peisar_ast/latest/peisar_ast/struct.Document.html
[visitor]: https://docs.rs/peisar_ast/latest/peisar_ast/trait.AstVisitor.html
