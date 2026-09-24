use peisar_ast::{AstOptions, md_to_ast, visit_document_mut, AstVisitor, VisitControl, Block};
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn highlighter(code: &str, lang: Option<String>) -> String {
    // Prefer the provided language token, fall back to plain text if unknown
    let lan = lang.as_deref().unwrap_or("text");
    let ss = SyntaxSet::load_defaults_newlines();
    let syntax = ss
        .find_syntax_by_token(lan)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let ts = ThemeSet::load_defaults();
    // Try to use a named theme, otherwise pick the first available theme as a fallback
    let theme = ts
        .themes
        .get("base16-ocean.dark")
        .cloned()
        .or_else(|| ts.themes.values().next().cloned())
        .expect("no theme available");

    highlighted_html_for_string(code, &ss, syntax, &theme)
        .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", escape_html(code)))
}

struct HighlightVisitor;

impl AstVisitor for HighlightVisitor {
    fn visit_block(&mut self, block: &mut Block) -> VisitControl {
        match block {
            Block::CodeBlock { lang, code, pos, attrs } => {
                // Create highlighted HTML and replace the code block with an HTML block
                let html = highlighter(code.as_str(), lang.clone());
                let new_node = Block::HtmlBlock {
                    html,
                    // Span is Copy
                    pos: *pos,
                    attrs: attrs.clone(),
                };
                VisitControl::replace_with(vec![new_node])
            }
            _ => VisitControl::keep_and_recurse(),
        }
    }
}

fn main() {
    let c = std::fs::read_to_string("packages/peisar_fs/README.md").unwrap();
    let mut doc = md_to_ast(
        &c,
        &AstOptions::default(),
        Some("packages/peisar_fs/README.md".to_string()),
    );

    let mut visitor = HighlightVisitor;
    visit_document_mut(&mut doc, &mut visitor);

    let json = serde_json::to_string_pretty(&doc).unwrap();
    std::fs::write("aa.json", json).ok();
}
