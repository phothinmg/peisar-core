use super::attrs::Attributes;
use super::span::Span;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Block-level nodes
// ---------------------------------------------------------------------------

/// Block-level nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    /// `# Heading` through `###### Heading`.
    Heading {
        level: u8,
        children: Vec<Inline>,
        pos: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },
    /// Plain paragraph text.
    Paragraph {
        children: Vec<Inline>,
        pos: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },
    /// Fenced or indented code block.
    CodeBlock {
        lang: Option<String>,
        code: String,
        pos: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },
    BlockQuote {
        children: Vec<Block>,
        pos: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },
    List {
        ordered: bool,
        items: Vec<ListItem>,
        pos: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },
    ThematicBreak {
        pos: Span,
    },
    HtmlBlock {
        html: String,
        pos: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },
    Table {
        table: Table,
        pos: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        attrs: Option<Attributes>,
    },
}

// ---------------------------------------------------------------------------
// Lists
// ---------------------------------------------------------------------------

/// A single list item (an `<li>`). Contains nested block content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListItem {
    pub children: Vec<Block>,
    /// GFM task-list state: `None` = not a task, `Some` = checked / unchecked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskState>,
    /// Span of the item in the *sub-document* of its enclosing list.
    pub pos: Span,
}

/// GFM task-list checkbox state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TaskState {
    /// `[ ]` — unchecked
    Unchecked,
    /// `[x]` / `[X]` — checked
    Checked,
}
// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

/// A GFM table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Table {
    pub header: TableRow,
    pub rows: Vec<TableRow>,
    /// Column alignment specifications (one per column).
    pub alignments: Vec<TableCellAlignment>,
}

/// A single table row (header or body).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

/// A single table cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableCell {
    pub children: Vec<Inline>,
}

/// Column alignment for table cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum TableCellAlignment {
    /// `:---` or `---` — default (left)
    #[default]
    Default,
    /// `:---`
    Left,
    /// `:---:`
    Center,
    /// `---:`
    Right,
}

// ---------------------------------------------------------------------------
// Inline-level nodes
// ---------------------------------------------------------------------------

/// Inline-level nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Inline {
    Text {
        value: String,
        pos: Span,
    },
    Emphasis {
        level: EmphasisLevel,
        children: Vec<Inline>,
        pos: Span,
    },
    Code {
        code: String,
        pos: Span,
    },
    HtmlInline {
        html: String,
        pos: Span,
    },
    /// GFM strikethrough: `~~text~~`.
    Strikethrough {
        children: Vec<Inline>,
        pos: Span,
    },
    HardBreak {
        pos: Span,
    },
    SoftBreak {
        pos: Span,
    },
    Image {
        alt: String,
        url: String,
        title: Option<String>,
        pos: Span,
    },
    Link {
        text: Vec<Inline>,
        url: String,
        title: Option<String>,
        autolink: bool,
        pos: Span,
    },
}

/// Emphasis strength.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EmphasisLevel {
    /// `*italic*` / `_italic_`
    Italic,
    /// `**bold**` / `__bold__`
    Bold,
}
