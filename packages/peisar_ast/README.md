<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->
<div align="center">
<img src="https://pub-c9ba018358dd48a99b70013b65a25e5f.r2.dev/logo/peisar.webp" width="160" height="160" alt="peisar" />
  <h1>Peisar AST</h1>
</div>

A lossless Markdown Abstract Syntax Tree (AST) parser with source-span
tracking, Kramdown block attributes, GFM extensions, YAML front-matter
extraction, and a visitor-based transformation API.

## Features

- **Lossless** — every node carries a [`Span`][span] so linters, language
  servers, and round-trip renderers can map back to the original source text.
- **GFM extensions** — tables, strikethrough, task lists, autolinks.
- **Kramdown block attributes** — `{:#id .class key="val"}` syntax on any
  block element.
- **YAML front-matter** — parsed and stripped automatically (via
  `peisar_frontmatter`); the deserialised value is available on the
  [`PeisarAst`][peisarast] struct.
- **Visitor API** — implement [`AstVisitor`][visitor] to inspect and mutate
  every block- and inline-level node with insert / replace / remove controls.
- **JSON output** — all node types implement `serde::Serialize` for
  straightforward JSON AST serialisation.
- **napi-rs ready** — every public type carries `#[cfg_attr(feature = "napi", napi::napi)]`
  annotations; enable the `napi` feature when building for Node.js.

## Installation

Add `peisar_ast` to your `Cargo.toml`:

```toml
[dependencies]
peisar_ast = { path = "../peisar_ast" }
```

For napi-rs / Node.js bindings, enable the `napi` feature:

```toml
peisar_ast = { path = "../peisar_ast", features = ["napi"] }
```

## Quick start

```rust
use peisar_ast::{PeisarAst, AstVisitor};

struct NoopVisitor;
impl AstVisitor for NoopVisitor {}

let md = "# Hello\n\nA paragraph with [a link](https://example.com).\n";
let mut ast = PeisarAst::<NoopVisitor>::new(md, None);

let doc = ast.ast();
assert_eq!(doc.children.len(), 2); // heading + paragraph
```

## Architecture

The crate is organised around three layers:

| Layer              | Modules                                                                  | Description                                                                                    |
| ------------------ | ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| **Tokens**         | `tokens::token`, `tokens::span`, `tokens::attrs`                         | The node types that make up the AST. Every node carries a `Span` for source-position tracking. |
| **Parsers**        | `parsers::block`, `parsers::inline`, `parsers::table`, `parsers::atters` | Block- and inline-level parsers that turn raw Markdown text into a `Document`.                 |
| **High-level API** | `PeisarAst`                                                              | Wraps a `Document` together with an optional front-matter value and a list of `AstVisitor`s.   |

### Block-level nodes (`Block`)

| Variant                   | Markdown                      | Notes                                   |
| ------------------------- | ----------------------------- | --------------------------------------- |
| `Heading`                 | `# H1` … `###### H6`          | Level 1–6, inline children              |
| `Paragraph`               | plain text                    | Inline children                         |
| `CodeBlock`               | ` ```rust … ``` ` or indented | `lang` from info string                 |
| `BlockQuote`              | `> …`                         | Children re-parsed as sub-document      |
| `List`                    | `1.`, `-`, `*`, `+`           | `ordered: bool`, `items: Vec<ListItem>` |
| `ThematicBreak`           | `---`, `***`, `___`           |                                         |
| `HtmlBlock`               | raw HTML                      |                                         |
| `Table`                   | GFM pipe table                | `Table { header, rows, alignments }`    |
| `LinkReferenceDefinition` | `[label]: url "title"`        | Collected at document level             |
| `Comment`                 | `<!-- … -->`                  | Markers stripped                        |

### Inline-level nodes (`Inline`)

| Variant         | Markdown                                          |
| --------------- | ------------------------------------------------- |
| `Text`          | plain text                                        |
| `Emphasis`      | `*italic*` / `**bold**` / `_italic_` / `__bold__` |
| `Code`          | `` `code` ``                                      |
| `HtmlInline`    | raw inline HTML                                   |
| `Strikethrough` | `~~text~~` (GFM)                                  |
| `HardBreak`     | two trailing spaces or `\` before newline         |
| `SoftBreak`     | ordinary newline within a paragraph               |
| `Image`         | `![alt](url "title")`                             |
| `Link`          | `[text](url "title")`, GFM autolinks              |
| `LinkReference` | `[text][label]`, `[label][]`, `[label]`           |

### Source positions (`Span` / `Position`)

Positions are **0-based**:

- `line` — line index (0 = first line)
- `column` — character column within that line
- `offset` — byte offset from the start of the input

A `Span` is a half-open range `[start, end)`.

### Kramdown attributes (`Attributes`)

```rust
use peisar_ast::tokens::Attributes;

let attrs = Attributes {
    id: Some("intro".into()),
    classes: Some(vec!["banner".into(), "wide".into()]),
    attributes: Some(vec![("data-index".into(), "42".into())],
};
assert_eq!(
    attrs.to_html_attr_string(),
    "id=\"intro\" class=\"banner wide\" data-index=\"42\""
);
```

## Parsing options (`AstOptions`)

```rust
use peisar_ast::AstOptions;

// Defaults: GFM + Kramdown both enabled.
let opts = AstOptions::default();

// Strict CommonMark only.
let opts = AstOptions {
    gfm: false,
    kramdown: false,
    file_name: None,
};
```

| Field       | Type             | Default | Description                                      |
| ----------- | ---------------- | ------- | ------------------------------------------------ |
| `gfm`       | `bool`           | `true`  | Tables, strikethrough, task lists, autolinks     |
| `kramdown`  | `bool`           | `true`  | Block attributes `{:#id .class key="val"}`       |
| `file_name` | `Option<String>` | `None`  | Optional source file name attached to `Document` |

## Visitor API

Implement `AstVisitor` to receive pre-order callbacks for every block and
inline node. The returned control value can request insertion, replacement,
or removal of nodes.

```rust
use peisar_ast::{PeisarAst, AstVisitor};
use peisar_ast::token::{Block, Inline};
use peisar_ast::parsers::visitor::{VisitControl, InlineVisitControl};

struct LinkCounter { count: usize }
impl AstVisitor for LinkCounter {
    fn visit_inline(&mut self, inline: &mut Inline) -> InlineVisitControl {
        if matches!(inline, Inline::Link { .. } | Inline::LinkReference { .. }) {
            self.count += 1;
        }
        InlineVisitControl::default()
    }
}

let md = "[a](http://x) and [b][ref]\n\n[ref]: http://y\n";
let mut ast = PeisarAst::<LinkCounter>::new(md, None);
ast.add_visitor(LinkCounter { count: 0 });
ast.visit_all();
```

### `VisitControl` / `InlineVisitControl`

| Field           | Effect                                             |
| --------------- | -------------------------------------------------- |
| `insert_before` | Insert nodes before the current node               |
| `insert_after`  | Insert nodes after the current node                |
| `replace_with`  | Replace the current node with the given nodes      |
| `remove`        | Remove the current node entirely                   |
| `recurse`       | Recurse into the node's children (default `false`) |

Convenience constructors: `keep_and_recurse()`, `remove()`, `replace_with(nodes)`.

## `PeisarAst` API

| Method                 | Description                                            |
| ---------------------- | ------------------------------------------------------ |
| `new(raw_md, options)` | Parse Markdown; front-matter is extracted and stripped |
| `add_visitor(visitor)` | Register a visitor (run in insertion order)            |
| `visit_all()`          | Run all registered visitors (pre-order)                |
| `clear_visitors()`     | Remove all visitors, keep AST                          |
| `take_visitors()`      | Take ownership of the visitor vector                   |
| `ast()` / `ast_mut()`  | Borrow the parsed `Document`                           |
| `take_ast()`           | Consume and return the `Document`                      |
| `frontmatter()`        | Borrow the parsed front-matter value (if any)          |
| `take_frontmatter()`   | Consume and return the front-matter value              |

The struct is generic over the visitor type `V` and the front-matter type `F`
(defaults to `serde_json::Value`). `F` must implement `serde::de::DeserializeOwned`.

## JSON serialisation

All node types derive `serde::Serialize`:

```rust
use peisar_ast::Document;
use serde_json::to_string_pretty;

let doc = Document::parse("# Title\n\nParagraph.\n", &Default::default(), None);
let json = to_string_pretty(&doc).unwrap();
```

`Block` and `Inline` use `#[serde(tag = "type", rename_all = "snake_case")]`,
so the JSON output mirrors mdast-compatible node names.

## License

Apache-2.0

[span]: src/tokens/span.rs
[peisarast]: src/lib.rs
[visitor]: src/parsers/visitor.rs
