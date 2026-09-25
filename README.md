<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->
<div align="center">
<img src="https://pub-c9ba018358dd48a99b70013b65a25e5f.r2.dev/logo/peisar.webp" width="160" height="160" alt="peisar" />
  <h1>Peisar</h1>
  <p>A lossless Markdown toolchain written in Rust — parse, transform, and render with source-span precision.</p>
</div>

---

Peisar Core is a Rust workspace of small, focused crates that together form a complete
Markdown processing pipeline: **parse** Markdown into a lossless AST (with GFM extensions
and Kramdown block attributes), **transform** it via a visitor-based API, and **render**
it to HTML — with optional YAML front-matter extraction and filesystem helpers.

Every AST node carries a [`Span`](#source-span-tracking) so you always know exactly where
it came from in the original source.

## Crates

| Crate                                       | Description                                                                 |
| ------------------------------------------- | --------------------------------------------------------------------------- |
| [`peisar_ast`](#peisar_ast)                 | Lossless Markdown AST parser with GFM, Kramdown attributes, and visitor API |
| [`peisar_frontmatter`](#peisar_frontmatter) | YAML front-matter extraction and deserialization                            |
| [`peisar_html`](#peisar_html)               | Render a `peisar_ast` document to HTML                                      |
| [`peisar_fs`](#peisar_fs)                   | Filesystem helper rooted at a configurable directory                        |
| [`peisar_log`](#peisar_log)                 | Colored, formatted logging utilities                                        |

**Dependency graph:**

```
peisar_log  ←──  peisar_fs
peisar_frontmatter  ←──  peisar_ast  ←──  peisar_html
```

## Quick Start

Add the crates you need to your `Cargo.toml`:

```toml
[dependencies]
peisar_ast = "0.1"
peisar_html = "0.1"
peisar_frontmatter = "0.1"
```

Parse Markdown, transform it, and render to HTML:

```rust
use peisar_ast::{PeisarAst, AstOptions, AstVisitor, Block, Inline, VisitControl};
use peisar_html::render_document_html;

// 1. Parse (front-matter is auto-extracted)
let mut ast = PeisarAst::<MyVisitor>::new(
    "---\ntitle: Hello\n---\n# Welcome\n\nSome **bold** text.",
    Some(AstOptions::default()),
);

// 2. Transform with a visitor
ast.add_visitor(MyVisitor);
ast.visit_all();

// 3. Render to HTML
let html = render_document_html(ast.ast(), None);
```

## `peisar_ast`

A lossless Markdown AST parser with source-span tracking, GFM extensions, Kramdown block
attributes, and a visitor-based transformation API.

### Features

- **Lossless** — every node carries a `Span` (start/end line, column, offset)
- **GFM extensions** — tables, strikethrough, task lists, autolinks
- **Kramdown block attributes** — `{:#id .class key="val"}` syntax
- **Visitor pattern** — pre-order traversal with insert / replace / remove / recurse control
- **Reference link resolution** — `[label]: url` definitions collected in a pre-pass
- **HTML comments** — parsed as `Block::Comment`
- **YAML front-matter** — automatically extracted and stripped
- **serde Serialize** — all AST types derive `Serialize` for JSON output

### `PeisarAst<V, F>` — high-level processor

Generic over a visitor type `V: AstVisitor` and front-matter type `F: DeserializeOwned`
(defaults to `serde_json::Value`).

| Method                                 | Description                                        |
| -------------------------------------- | -------------------------------------------------- |
| `new(raw_md, options)`                 | Parse Markdown, extract front-matter, build AST    |
| `add_visitor(visitor)`                 | Register a visitor (run in insertion order)        |
| `visit_all()`                          | Run all visitors in pre-order traversal            |
| `ast()` / `ast_mut()`                  | Borrow the `Document` (auto-runs pending visitors) |
| `take_ast()`                           | Consume and return the `Document`                  |
| `frontmatter()` / `take_frontmatter()` | Access extracted front-matter                      |

### `AstOptions`

```rust
pub struct AstOptions {
    pub gfm: bool,              // GFM extensions. Default: true
    pub kramdown: bool,         // Kramdown block attributes. Default: true
    pub file_name: Option<String>,
}
```

### `Document` — AST root

```rust
pub struct Document {
    pub node_type: String,      // "root"
    pub file_name: Option<String>,
    pub pos: Span,
    pub children: Vec<Block>,
    pub link_references: Vec<LinkReferenceDefinition>,
}
```

### Block nodes

`Heading`, `Paragraph`, `CodeBlock`, `BlockQuote`, `List`, `ThematicBreak`, `HtmlBlock`,
`Table`, `LinkReferenceDefinition`, `Comment` — all carry `pos: Span`, most carry
`attrs: Option<Attributes>`.

### Inline nodes

`Text`, `Emphasis` (`Italic` / `Bold`), `Code`, `HtmlInline`, `Strikethrough`, `HardBreak`,
`SoftBreak`, `Image`, `Link`, `LinkReference` — all carry `pos: Span`.

### Source span tracking

```rust
pub struct Position { pub line: usize, pub column: usize, pub offset: usize }
pub struct Span { pub start: Position, pub end: Position }
```

### Visitor API

Implement the `AstVisitor` trait to inspect and mutate nodes during a pre-order traversal:

```rust
use peisar_ast::{AstVisitor, Block, Inline, VisitControl, InlineVisitControl};

struct MyVisitor;

impl AstVisitor for MyVisitor {
    fn visit_block(&mut self, block: &mut Block) -> VisitControl {
        match block {
            Block::Heading { level, .. } if *level == 1 => {
                // Replace all H1s with an H2
                VisitControl::replace_with(vec![/* ... */])
            }
            _ => VisitControl::keep_and_recurse(),
        }
    }

    fn visit_inline(&mut self, _inline: &mut Inline) -> InlineVisitControl {
        InlineVisitControl::default()
    }
}
```

`VisitControl` fields:

| Field           | Effect                                     |
| --------------- | ------------------------------------------ |
| `insert_before` | Insert blocks before the current node      |
| `insert_after`  | Insert blocks after the current node       |
| `replace_with`  | Replace the current node with these blocks |
| `remove`        | Remove the current node                    |
| `recurse`       | Descend into children (default: `false`)   |

### Kramdown attributes

```rust
pub struct Attributes {
    pub id: Option<String>,
    pub classes: Option<Vec<String>>,
    pub attributes: Option<Vec<(String, String)>>,
}
```

Use `Attributes::to_html_attr_string()` to render as an HTML attribute string.

### Standalone parser functions

```rust
use peisar_ast::{md_to_ast, Document, AstOptions};

let doc = md_to_ast("# Hello\n\nWorld", &AstOptions::default(), None);
// or
let doc = Document::parse("# Hello", &AstOptions::default(), Some("readme.md".into()));
```

Inline-only parsing:

```rust
use peisar_ast::parsers::inline::parse_inline;
let inlines = parse_inline("some **bold** text", None);
```

## `peisar_frontmatter`

Parse optional YAML front matter from Markdown documents and deserialize it into any
serde-enabled type.

```rust
use peisar_frontmatter::parse_markdown_frontmatter;
use serde::Deserialize;

#[derive(Deserialize)]
struct Frontmatter { title: String, draft: bool }

let content = "---\ntitle: Hello\ndraft: false\n---\n# Welcome\n";
let parsed = parse_markdown_frontmatter::<Frontmatter>(content).unwrap();

assert_eq!(parsed.yaml_data().unwrap().title, "Hello");
assert_eq!(parsed.pure_markdown_content(), "# Welcome\n");

// Or consume into owned parts:
let (markdown, frontmatter) = parsed.into_parts();
```

`ParseResult<T>` methods: `yaml_data()`, `pure_markdown_content()`, `into_parts()`.

## `peisar_html`

Render a `peisar_ast` `Document` into HTML.

```rust
use peisar_ast::{Document, AstOptions};
use peisar_html::{render_document_html, AstToHtml, RenderOptions};

let doc = Document::parse("# Hello\n\nWorld", &AstOptions::default(), None);

// Fragment mode (body only) — default
let fragment = render_document_html(&doc, None);

// Full HTML document
let opts = RenderOptions {
    fragment: false,
    title: Some("My Page".into()),
    body_class: Some("markdown-body".into()),
    style: Some("body { font-family: sans-serif; }".into()),
    ..Default::default()
};
let full_html = AstToHtml::new(Some(opts)).render_document_owned(&doc);
```

`RenderOptions` fields:

| Field        | Default | Description                                                   |
| ------------ | ------- | ------------------------------------------------------------- |
| `fragment`   | `true`  | `true` = body only; `false` = full `<!DOCTYPE html>` document |
| `charset`    | `true`  | Emit `<meta charset="utf-8">`                                 |
| `viewport`   | `true`  | Emit `<meta name="viewport" …>`                               |
| `title`      | `None`  | `<title>` content                                             |
| `body_class` | `None`  | CSS classes on `<body>`                                       |
| `style`      | `None`  | Inline CSS in a `<style>` block                               |

`AstToHtml` also exposes `render_block()`, `render_inline()`, and `finish()` for
incremental rendering.

Run the bundled example:

```sh
cargo run -p peisar_html --example basic
```

## `peisar_fs`

Filesystem helper for performing common file operations relative to a configured root
directory.

```rust
use peisar_fs::PeisarFs;

let fs = PeisarFs::new(Some("/tmp/myproject".into()));

fs.mkdir("src/components".into());
fs.write_file("src/components/app.rs".into(), "fn main() {}".into());

let content = fs.read_file("src/components/app.rs".into());
let rust_files = fs.read_dir("src".into(), Some("rs".into()));
```

| Method                      | Description                                              |
| --------------------------- | -------------------------------------------------------- |
| `new(root)`                 | Create helper rooted at `root` (defaults to current dir) |
| `exists(path)`              | Check if path exists below root                          |
| `mkdir(path)`               | Create directory + all parents                           |
| `write_file(path, content)` | Write file, creating parent dirs                         |
| `read_file(path)`           | Read UTF-8 text                                          |
| `read_dir(path, ext)`       | Recursively list files, optionally filtered by extension |

## `peisar_log`

Colored, formatted logging utilities for the Peisar project.

```rust
peisar_log::info("Operation completed");
peisar_log::warning("Deprecated feature used");
peisar_log::error("Failed to read file", true); // true = exit(1)
```

| Function           | Color   | Output                                           |
| ------------------ | ------- | ------------------------------------------------ |
| `info(msg)`        | green   | `[Peisar Info] msg`                              |
| `warning(msg)`     | yellow  | `[Peisar Warning] msg`                           |
| `error(msg, exit)` | magenta | `[Peisar Error] msg` (exits if `exit` is `true`) |

## Node.js / napi-rs Support

Every crate has an optional `napi` feature that enables [napi-rs](https://napi.rs)
derive macros on all public types, allowing them to be compiled into a native Node.js
addon:

```toml
[dependencies]
peisar_ast = { version = "0.1", features = ["napi"] }
```

The `peisar_ast` napi layer exposes `PeisarAstJs` — a JS-friendly mirror that accepts
visitor callbacks and returns the AST as a JSON string.

## Development

```sh
make check    # Cross-compile cargo check for 8 targets (linux/musl/apple/windows × x86_64/aarch64)
make test     # cargo test
make fmt      # cargo fmt
make doc      # cargo doc --open
make publish  # cargo publish --workspace
```

## License

Apache-2.0
