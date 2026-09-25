//! Callback-based wrapper layer for JavaScript (napi-rs) interop.
//!
//! The Rust [`AstVisitor`](crate::AstVisitor) trait cannot be exported
//! directly through napi-rs (traits and generics are not supported).
//! This module provides a parallel, callback-based API that JS consumers
//! use instead:
//!
//! 1. [`VisitControlJs`] / [`InlineVisitControlJs`] — napi-exported
//!    mirror structs that JS returns from its callback functions.
//! 2. [`BlockCallback`] / [`InlineCallback`] — type-erased napi callbacks
//!    (`ThreadsafeFunction` wrappers).
//! 3. [`JsVisitor`] — adapter that implements [`AstVisitor`] by forwarding
//!    to the JS callbacks.
//! 4. [`PeisarAstJs`] — napi-exported high-level struct mirroring
//!    [`PeisarAst`](crate::PeisarAst) but accepting JS callbacks instead of
//!    Rust visitor types.
//!
//! ## JS usage example
//!
//! ```js
//! const { PeisarAstJs } = require("@peisar/ast");
//!
//! const ast = new PeisarAstJs("# Hello\n\nA paragraph.", undefined);
//!
//! ast.addVisitor({
//!   visitBlock(block) {
//!     if (block.type === "heading") console.log("heading:", block.level);
//!     return { recurse: true };
//!   },
//!   visitInline(inline) {
//!     return {}; // keep, no recurse
//!   },
//! });
//!
//! ast.visitAll();
//! console.log(JSON.stringify(ast.ast, null, 2));
//! ```

use crate::parsers::visitor::{AstVisitor, InlineVisitControl, VisitControl, visit_document_mut};
use crate::tokens::token::{Block, Inline};
use crate::{AstOptions, Document};

// ---------------------------------------------------------------------------
// napi-exported control mirrors
// ---------------------------------------------------------------------------

/// JS-facing mirror of [`VisitControl`].
///
/// Returned from the JS `visitBlock` callback.  All fields are optional;
/// omitting a field means "no change" for that operation.
#[cfg_attr(feature = "napi", napi::napi(object))]
#[derive(Debug, Default, Clone)]
pub struct VisitControlJs {
    /// Nodes to insert before the current node.
    pub insert_before: Option<Vec<Block>>,
    /// Nodes to insert after the current node.
    pub insert_after: Option<Vec<Block>>,
    /// Replace the current node with these nodes.
    pub replace_with: Option<Vec<Block>>,
    /// Remove the current node entirely.
    pub remove: Option<bool>,
    /// Whether to recurse into this node's children.
    pub recurse: Option<bool>,
}

impl From<VisitControlJs> for VisitControl {
    fn from(js: VisitControlJs) -> Self {
        VisitControl {
            insert_before: js.insert_before.unwrap_or_default(),
            insert_after: js.insert_after.unwrap_or_default(),
            replace_with: js.replace_with,
            remove: js.remove.unwrap_or(false),
            recurse: js.recurse.unwrap_or(false),
        }
    }
}

/// JS-facing mirror of [`InlineVisitControl`].
///
/// Returned from the JS `visitInline` callback.  All fields are optional;
/// omitting a field means "no change" for that operation.
#[cfg_attr(feature = "napi", napi::napi(object))]
#[derive(Debug, Default, Clone)]
pub struct InlineVisitControlJs {
    /// Nodes to insert before the current node.
    pub insert_before: Option<Vec<Inline>>,
    /// Nodes to insert after the current node.
    pub insert_after: Option<Vec<Inline>>,
    /// Replace the current node with these nodes.
    pub replace_with: Option<Vec<Inline>>,
    /// Remove the current node entirely.
    pub remove: Option<bool>,
    /// Whether to recurse into this node's children.
    pub recurse: Option<bool>,
}

impl From<InlineVisitControlJs> for InlineVisitControl {
    fn from(js: InlineVisitControlJs) -> Self {
        InlineVisitControl {
            insert_before: js.insert_before.unwrap_or_default(),
            insert_after: js.insert_after.unwrap_or_default(),
            replace_with: js.replace_with,
            remove: js.remove.unwrap_or(false),
            recurse: js.recurse.unwrap_or(false),
        }
    }
}

// ---------------------------------------------------------------------------
// Type-erased callbacks (napi ThreadsafeFunction wrappers)
// ---------------------------------------------------------------------------

/// Type-erased block-visitor callback.
///
/// Under the `napi` feature this is a `ThreadsafeFunction`; without the
/// feature it is a zero-sized type so the crate still compiles in
/// pure-Rust mode.
#[cfg(feature = "napi")]
pub type BlockCallback = napi::threadsafe_function::ThreadsafeFunction<Block, VisitControlJs>;

#[cfg(not(feature = "napi"))]
pub type BlockCallback = std::marker::PhantomData<fn(Block) -> VisitControlJs>;

/// Type-erased inline-visitor callback.
#[cfg(feature = "napi")]
pub type InlineCallback =
    napi::threadsafe_function::ThreadsafeFunction<Inline, InlineVisitControlJs>;

#[cfg(not(feature = "napi"))]
pub type InlineCallback = std::marker::PhantomData<fn(Inline) -> InlineVisitControlJs>;

// ---------------------------------------------------------------------------
// JsVisitor — adapter implementing AstVisitor via JS callbacks
// ---------------------------------------------------------------------------

/// Adapter that implements [`AstVisitor`] by forwarding to JS callbacks.
///
/// Created internally by [`PeisarAstJs`]; users do not construct this
/// directly.
#[derive(Default)]
pub struct JsVisitor {
    /// Optional JS callback for block nodes.
    pub block_cb: Option<BlockCallback>,
    /// Optional JS callback for inline nodes.
    pub inline_cb: Option<InlineCallback>,
}

impl AstVisitor for JsVisitor {
    fn visit_block(&mut self, block: &mut Block) -> VisitControl {
        match &self.block_cb {
            #[cfg(feature = "napi")]
            Some(cb) => {
                // Hand a clone of the current block to JS; JS may return a
                // control struct with replacements.
                let snapshot = block.clone();
                let js_ctrl: VisitControlJs = cb.call(
                    Ok(snapshot),
                    napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
                );
                js_ctrl.into()
            }
            #[cfg(not(feature = "napi"))]
            Some(_) => VisitControl::default(),
            None => VisitControl::default(),
        }
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> InlineVisitControl {
        match &self.inline_cb {
            #[cfg(feature = "napi")]
            Some(cb) => {
                let snapshot = inline.clone();
                let js_ctrl: InlineVisitControlJs = cb.call(
                    Ok(snapshot),
                    napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
                );
                js_ctrl.into()
            }
            #[cfg(not(feature = "napi"))]
            Some(_) => InlineVisitControl::default(),
            None => InlineVisitControl::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// PeisarAstJs — napi-exported high-level API
// ---------------------------------------------------------------------------

/// napi-exported high-level Markdown AST processor.
///
/// Mirrors [`PeisarAst`](crate::PeisarAst) but uses JS callbacks instead
/// of Rust visitor types.  JS consumers create this, register visitor
/// objects with [`add_visitor`], then call [`visit_all`].
///
/// The AST document and front-matter getters automatically run
/// [`visit_all`] first when there are registered visitors, so JS consumers
/// always see a post-visit AST.
///
/// # JS example
///
/// ```js
/// const ast = new PeisarAstJs("# Hello", undefined);
/// ast.addVisitor({
///   visitBlock(b) { console.log(b.type); return { recurse: true }; },
/// });
/// // No explicit visitAll() needed — ast / frontmatter getters run it.
/// console.log(JSON.stringify(ast.astJson, null, 2));
/// console.log(ast.frontmatter);
/// ```
///
/// [`add_visitor`]: Self::add_visitor
/// [`visit_all`]: Self::visit_all
#[cfg_attr(feature = "napi", napi::napi)]
pub struct PeisarAstJs {
    /// The parsed AST document (private — use [`ast_json`][Self::get_ast_json]
    /// or [`frontmatter`][Self::get_frontmatter] getters which auto-run
    /// visitors).
    ast: Document,
    /// Registered JS visitors (adapter wrappers).
    visitors: Vec<JsVisitor>,
    /// Parsed YAML front-matter (if any).
    frontmatter: Option<serde_json::Value>,
}

#[cfg_attr(feature = "napi", napi::napi)]
impl PeisarAstJs {
    /// Create a new `PeisarAstJs` from raw Markdown text.
    ///
    /// If `options` is `None`, defaults are used (GFM + Kramdown enabled).
    #[cfg_attr(feature = "napi", napi::constructor)]
    pub fn new(raw_md: String, options: Option<AstOptions>) -> Self {
        let opts = options.unwrap_or_else(AstOptions::default);
        let f_n = opts.file_name.clone();

        // Parse front matter (reuse the same logic as PeisarAst).
        let (md_content, frontmatter) =
            match peisar_frontmatter::parse_markdown_frontmatter::<serde_json::Value>(&raw_md) {
                Ok(parsed) => parsed.into_parts(),
                Err(_) => (raw_md.clone(), None),
            };

        let ast = crate::parsers::md_to_ast(&md_content, &opts, f_n);

        Self {
            ast,
            visitors: Vec::new(),
            frontmatter,
        }
    }

    /// Register a JS visitor.
    ///
    /// `block_cb` and `inline_cb` are JS functions.  Either may be
    /// `undefined`/`null` to skip that node kind.  The JS function
    /// receives a `Block` or `Inline` and returns a
    /// `VisitControlJs` / `InlineVisitControlJs` (or `undefined`).
    #[cfg_attr(feature = "napi", napi)]
    pub fn add_visitor(
        &mut self,
        block_cb: Option<BlockCallback>,
        inline_cb: Option<InlineCallback>,
    ) {
        self.visitors.push(JsVisitor {
            block_cb,
            inline_cb,
        });
    }

    /// Run all registered visitors in insertion order.
    ///
    /// This is called automatically by the [`ast_json`][Self::get_ast_json]
    /// and [`frontmatter`][Self::get_frontmatter] getters when there are
    /// registered visitors, but can also be called manually.
    #[cfg_attr(feature = "napi", napi)]
    pub fn visit_all(&mut self) {
        if self.visitors.is_empty() {
            return;
        }
        for v in &mut self.visitors {
            visit_document_mut(&mut self.ast, v);
        }
    }

    /// Remove all registered visitors.
    #[cfg_attr(feature = "napi", napi)]
    pub fn clear_visitors(&mut self) {
        self.visitors.clear();
    }

    /// Borrow the parsed AST document.
    ///
    /// Runs [`visit_all`](Self::visit_all) first if there are registered
    /// visitors, so the returned AST always reflects visitor mutations.
    #[cfg_attr(feature = "napi", napi)]
    pub fn ast(&mut self) -> &Document {
        if !self.visitors.is_empty() {
            self.visit_all();
        }
        &self.ast
    }

    /// Get a JSON-serializable copy of the AST document.
    ///
    /// Runs [`visit_all`](Self::visit_all) first if there are registered
    /// visitors.
    #[cfg_attr(feature = "napi", napi(getter))]
    pub fn get_ast_json(&mut self) -> serde_json::Value {
        if !self.visitors.is_empty() {
            self.visit_all();
        }
        serde_json::to_value(&self.ast).unwrap_or(serde_json::Value::Null)
    }

    /// Get the parsed YAML front-matter (if any).
    ///
    /// Runs [`visit_all`](Self::visit_all) first if there are registered
    /// visitors.
    #[cfg_attr(feature = "napi", napi(getter))]
    pub fn get_frontmatter(&mut self) -> Option<serde_json::Value> {
        if !self.visitors.is_empty() {
            self.visit_all();
        }
        self.frontmatter.clone()
    }
}
