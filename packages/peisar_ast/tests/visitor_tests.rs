use peisar_ast::{md_to_ast, visit_document_mut, AstOptions, AstVisitor, VisitControl, InlineVisitControl};
use peisar_ast::tokens::token::{Block, Inline};
use peisar_ast::tokens::span::Span;

struct ExampleVisitor;

impl AstVisitor for ExampleVisitor {
    fn visit_block(&mut self, block: &mut Block) -> VisitControl {
        match block {
            Block::ThematicBreak { .. } => VisitControl::remove(),
            Block::CodeBlock { .. } => {
                // Insert a short note paragraph before each code block
                let note = Block::Paragraph {
                    children: vec![Inline::Text {
                        value: "Note: code follows".to_string(),
                        pos: Span::default(),
                    }],
                    pos: Span::default(),
                    attrs: None,
                };
                let mut c = VisitControl::default();
                c.insert_before.push(note);
                c
            }
            _ => VisitControl::keep_and_recurse(),
        }
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> InlineVisitControl {
        if let Inline::Text { value, .. } = inline {
            if value.contains("foo") {
                *value = value.replace("foo", "FOO");
            }
        }
        InlineVisitControl::keep_and_recurse()
    }
}

#[test]
fn test_visit_replace_insert() {
    let src = "# Heading 1\n\nSome foo text\n\n---\n\n```rust\nfn main() {}\n```\n";
    let mut doc = md_to_ast(src, &AstOptions::default(), None);
    let mut visitor = ExampleVisitor;
    visit_document_mut(&mut doc, &mut visitor);

    // Ensure there are no thematic break nodes left
    assert!(doc.children.iter().all(|c| !matches!(c, Block::ThematicBreak { .. })));

    // Find the code block index
    let code_idx = doc
        .children
        .iter()
        .position(|c| matches!(c, Block::CodeBlock { .. }))
        .expect("code block not found");
    assert!(code_idx > 0, "code block should not be the first node");

    // The node immediately before the code block should be the inserted note paragraph
    match &doc.children[code_idx - 1] {
        Block::Paragraph { children, .. } => {
            let found = children.iter().any(|inl| match inl {
                Inline::Text { value, .. } => value == "Note: code follows",
                _ => false,
            });
            assert!(found, "Expected an inserted note paragraph before code block");
        }
        other => panic!("Expected paragraph before code block, found: {:?}", other),
    }

    // Find the (original) paragraph that contained "Some foo text" and ensure it now contains "FOO"
    let para_with_foo = doc.children.iter().find(|c| match c {
        Block::Paragraph { children, .. } => children.iter().any(|inl| match inl {
            Inline::Text { value, .. } => value.contains("FOO"),
            _ => false,
        }),
        _ => false,
    });
    assert!(para_with_foo.is_some(), "Expected a paragraph containing FOO after visit");
}
