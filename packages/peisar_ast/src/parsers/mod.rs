mod atters;
mod block;
mod inline;
mod table;
pub mod visitor;

use crate::options::AstOptions;
use crate::tokens::{span::Span, token::Block};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Document
// ---------------------------------------------------------------------------

/// A Markdown document — the root of the AST.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Document {
    #[serde(rename = "type")]
    pub node_type: &'static str,
    pub file_name: Option<String>,
    /// Span of the whole document in the source text.
    pub pos: Span,
    pub children: Vec<Block>,
}
/// Parse a Markdown string into a [`Document`] with the given options.
pub fn md_to_ast(input: &str, opts: &AstOptions, file_name: Option<String>) -> Document {
    let line_starts = block::compute_line_starts(input);
    let lines: Vec<&str> = input.lines().collect();
    let mut p = block::ParserState::new(input, &lines, &line_starts, opts, file_name.clone());
    let start = p.position_at(0);

    let mut blocks = Vec::new();

    while !p.is_done() {
        if p.skip_blank_lines() {
            continue;
        }
        if let Some(block) = p.parse_block() {
            blocks.push(block);
        } else {
            p.advance(); // safety net — never loop forever
        }
    }

    let end = p.position_at(p.pos.min(p.lines.len()));

    Document {
        node_type: "root",
        pos: Span::new(start, end),
        file_name,
        children: blocks,
    }
}
