//! Source position tracking for AST nodes.
//!
//! Every node in the AST carries a [`Span`] so that tools (linters,
//! language servers, round-trip renderers) can map back to the original
//! source text.
//!
//! Positions are **0-based**:
//! - `line` is the line index (0 = first line),
//! - `column` is the character column within that line,
//! - `offset` is the byte offset from the start of the input.

use serde::Serialize;

#[cfg(feature = "napi")]
use napi_derive::napi;

/// Under the `napi` feature `usize` is not a supported N-API type, so we
/// expose the integer fields as `u32` instead. In pure-Rust builds `u32`
/// and `usize` are kept separate via this alias to avoid silent truncation.
#[cfg(not(feature = "napi"))]
pub type Index = usize;
#[cfg(feature = "napi")]
pub type Index = u32;

/// A zero-based point in the source text.
#[cfg_attr(feature = "napi", napi(object))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct Position {
    /// Line number, 0-based.
    pub line: Index,
    /// Column number, 0-based (in characters).
    pub column: Index,
    /// Byte offset from the start of the input.
    pub offset: Index,
}

/// A half-open span `[start, end)` covering a node's source text.
///
/// Both `start` and `end` are inclusive [`Position`]s that point into the
/// original source string.
#[cfg_attr(feature = "napi", napi(object))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct Span {
    /// The start position (inclusive).
    pub start: Position,
    /// The end position (exclusive).
    pub end: Position,
}

impl Span {
    /// Create a new span from `start` to `end`.
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl Position {
    /// Create a new position at the given `line`, `column`, and byte `offset`.
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line: line.try_into().unwrap(),
            column: column.try_into().unwrap(),
            offset: offset.try_into().unwrap(),
        }
    }
}
