mod options;
mod parsers;

pub mod tokens;
mod utils;

// Re-export core AST node types for consumers/tests
pub use options::AstOptions;
pub use parsers::Document;
pub use parsers::md_to_ast;
pub use tokens::span::Span;
pub use tokens::token::{Block, Inline};

// Visitor helpers for mutating/traversing the AST
pub use parsers::visitor::{AstVisitor, InlineVisitControl, VisitControl, visit_document_mut};
use peisar_frontmatter::parse_markdown_frontmatter;
use serde_json::Value;

pub struct PeisarAst<V: AstVisitor, F = Value> {
    ast: Document,
    visitors: Vec<V>,
    frontmatter: Option<F>,
}

impl<V: AstVisitor, F> PeisarAst<V, F>
where
    F: serde::de::DeserializeOwned,
{
    pub fn new(raw_md: &str, options: Option<AstOptions>, file_name: Option<String>) -> Self {
        let opts = options.unwrap_or_else(AstOptions::default);
        let f_n: Option<String> = if file_name.is_none() { None } else { file_name };

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

    /// Add a visitor of the concrete visitor type V.
    pub fn add_visitor(&mut self, visitor: V) {
        self.visitors.push(visitor);
    }

    /// Run all registered visitors (in insertion order) against the AST.
    pub fn visit_all(&mut self) {
        for v in &mut self.visitors {
            visit_document_mut(&mut self.ast, v);
        }
    }

    /// Remove all visitors but keep the AST intact.
    pub fn clear_visitors(&mut self) {
        self.visitors.clear();
    }

    /// Take ownership of the visitor vector, leaving an empty Vec in its place.
    pub fn take_visitors(&mut self) -> Vec<V> {
        std::mem::take(&mut self.visitors)
    }

    /// Borrow the parsed frontmatter (if any).
    pub fn frontmatter(&self) -> Option<&F> {
        self.frontmatter.as_ref()
    }

    /// Consume and return the frontmatter value, leaving None in its place.
    pub fn take_frontmatter(&mut self) -> Option<F> {
        self.frontmatter.take()
    }
}
