//! # peisar_ast
//!
//! A lossless Markdown Abstract Syntax Tree (AST) parser with source-span
//! tracking, Kramdown block attributes, GFM extensions, YAML front-matter
//! extraction, and a visitor-based transformation API.
//!
//! ## Overview
//!
//! The crate is organised around three layers:
//!
//! 1. **Tokens** ([`tokens::token`], [`Span`](tokens::span::Span),
//!    [`Attributes`](tokens::Attributes)) —
//!    the node types that make up the AST.  Every node carries a [`Span`]
//!    so that tools (linters, language servers, round-trip renderers) can
//!    map back to the original source text.
//! 2. **Parsers** ([`parsers`]) — the block- and inline-level parsers that
//!    turn raw Markdown text into a [`Document`].
//! 3. **High-level API** ([`PeisarAst`]) — wraps a [`Document`] together with
//!    an optional front-matter value and a list of [`AstVisitor`]s.
//!
//! ## Quick example
//!
//! ```rust
//! use peisar_ast::{PeisarAst, AstVisitor};
//!
//! struct NoopVisitor;
//! impl AstVisitor for NoopVisitor {}
//!
//! let md = "# Hello\n\nA paragraph with [a link](https://example.com).\n";
//! let mut ast = PeisarAst::<NoopVisitor>::new(md, None);
//!
//! // Inspect the parsed AST.
//! let doc = ast.ast();
//! assert_eq!(doc.children.len(), 2); // heading + paragraph
//! ```
//!
//! ## napi-rs compatibility
//!
//! All public types carry `#[cfg_attr(feature = "napi", napi::napi)]`
//! annotations.  Enable the `napi` feature when building for Node.js from
//! the parent `peisar` workspace:
//!
//! ```toml
//! peisar_ast = { path = "../peisar_ast", features = ["napi"] }
//! ```
//!
//! [`Span`]: tokens::span::Span

#![warn(missing_docs)]

mod nodes;
mod options;
mod parsers;
mod tokens;

#[cfg(feature = "napi")]
pub mod js;

// Re-export
pub use nodes::{Document, token};
pub use options::AstOptions;
pub use parsers::visitor::AstVisitor;
pub use tokens::Attributes;
pub use tokens::token::LinkReferenceDefinition;

// Re-export JS callback-based API (available under `napi` feature).
#[cfg(feature = "napi")]
pub use js::{
    BlockCallback, InlineCallback, InlineVisitControlJs, JsVisitor, PeisarAstJs, VisitControlJs,
};
// Use for PeisarAst
use parsers::md_to_ast;
use parsers::visitor::visit_document_mut;
use peisar_frontmatter::parse_markdown_frontmatter;
use serde_json::Value;

/// The high-level Markdown AST processor.
///
/// Wraps a parsed [`Document`] together with an optional front-matter value
/// and a list of [`AstVisitor`]s.  Generic over the visitor type `V` and the
/// front-matter type `F` (defaults to [`serde_json::Value`]).
///
/// # Type parameters
///
/// - `V` — the concrete [`AstVisitor`] implementation.  All registered
///   visitors must share the same type.
/// - `F` — the deserialised front-matter type.  Must implement
///   [`serde::de::DeserializeOwned`].  Defaults to [`serde_json::Value`].
///
/// # Example
///
/// ```rust
/// use peisar_ast::{PeisarAst, AstVisitor};
///
/// struct NoopVisitor;
/// impl AstVisitor for NoopVisitor {}
///
/// let md = "# Title\n\nBody text.\n";
/// let mut ast = PeisarAst::<NoopVisitor>::new(md, None);
/// assert_eq!(ast.ast().children.len(), 2);
/// ```
pub struct PeisarAst<V: AstVisitor, F = Value> {
    ast: Document,
    visitors: Vec<V>,
    frontmatter: Option<F>,
}

impl<V: AstVisitor, F> PeisarAst<V, F>
where
    F: serde::de::DeserializeOwned,
{
    /// Create a new `PeisarAst` from raw Markdown text.
    ///
    /// If `options` is `None`, [`AstOptions::default`] is used (GFM and
    /// Kramdown both enabled).
    ///
    /// Any YAML front-matter present at the top of the document is parsed
    /// and stripped from the Markdown body.
    pub fn new(raw_md: &str, options: Option<AstOptions>) -> Self {
        let opts = options.unwrap_or_else(AstOptions::default);
        let f_n: Option<String> = if opts.file_name.is_none() {
            None
        } else {
            opts.clone().file_name
        };

        // Parse and strip any YAML front matter and keep the owned frontmatter
        // value (if present) as `frontmatter`.
        let (md_content, frontmatter) = match parse_markdown_frontmatter::<F>(raw_md) {
            Ok(parsed) => parsed.into_parts(),
            Err(_) => (raw_md.to_string(), None),
        };

        let ast = md_to_ast(&md_content, &opts, f_n);

        Self {
            ast,
            visitors: Vec::new(),
            frontmatter,
        }
    }

    /// Add a visitor of the concrete visitor type `V`.
    ///
    /// Visitors are run in insertion order when [`visit_all`](Self::visit_all)
    /// is called.
    pub fn add_visitor(&mut self, visitor: V) {
        self.visitors.push(visitor);
    }

    /// Run all registered visitors (in insertion order) against the AST.
    ///
    /// Each visitor receives mutable access to every block- and inline-level
    /// node via pre-order traversal.
    pub fn visit_all(&mut self) {
        for v in &mut self.visitors {
            visit_document_mut(&mut self.ast, v);
        }
    }

    /// Remove all visitors but keep the AST intact.
    pub fn clear_visitors(&mut self) {
        self.visitors.clear();
    }

    /// Take ownership of the visitor vector, leaving an empty [`Vec`] in its place.
    pub fn take_visitors(&mut self) -> Vec<V> {
        std::mem::take(&mut self.visitors)
    }

    /// Borrow the parsed AST.
    ///
    /// Runs [`visit_all`](Self::visit_all) first when there are registered
    /// visitors, so the returned AST always reflects visitor mutations.
    pub fn ast(&mut self) -> &Document {
        if !self.visitors.is_empty() {
            self.visit_all();
        }
        &self.ast
    }

    /// Borrow the parsed AST mutably.
    ///
    /// Runs [`visit_all`](Self::visit_all) first when there are registered
    /// visitors.
    pub fn ast_mut(&mut self) -> &mut Document {
        if !self.visitors.is_empty() {
            self.visit_all();
        }
        &mut self.ast
    }

    /// Consume and return the AST, leaving a default [`Document`] in its place.
    ///
    /// Runs [`visit_all`](Self::visit_all) first when there are registered
    /// visitors.
    pub fn take_ast(&mut self) -> Document {
        if !self.visitors.is_empty() {
            self.visit_all();
        }
        std::mem::take(&mut self.ast)
    }

    /// Borrow the parsed front-matter (if any).
    ///
    /// Runs [`visit_all`](Self::visit_all) first when there are registered
    /// visitors.
    pub fn frontmatter(&mut self) -> Option<&F> {
        if !self.visitors.is_empty() {
            self.visit_all();
        }
        self.frontmatter.as_ref()
    }

    /// Consume and return the front-matter value, leaving `None` in its place.
    ///
    /// Runs [`visit_all`](Self::visit_all) first when there are registered
    /// visitors.
    pub fn take_frontmatter(&mut self) -> Option<F> {
        if !self.visitors.is_empty() {
            self.visit_all();
        }
        self.frontmatter.take()
    }
}

#[cfg(test)]
mod tests;
