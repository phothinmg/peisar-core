use peisar_ast::{AstOptions, md_to_ast};
// use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

// struct Highlighter {
//     html: String,
//     positions: span::Span,
//     ast: nodes::Document,
// }

// impl visitor::AstNodeVisitor for Highlighter {
//     fn visit_document(&mut self, doc: &nodes::Document) {
//         self.ast = doc.clone();
//     }
//     fn visit_code_block(
//         &mut self,
//         lang: Option<&str>,
//         code: &str,
//         _attrs: Option<&nodes::KramdownAttributes>,
//     ) {
//         let lan = lang.unwrap_or("text");
//         let syntax_set = SyntaxSet::load_defaults_newlines();
//         let syntax_reference = syntax_set.find_syntax_by_token(lan).unwrap();
//         let theme = ThemeSet::load_defaults().themes["base16-ocean.dark"].clone();
//     }
// }

fn main() {
    let c = std::fs::read_to_string("packages/peisar_fs/README.md").unwrap();
    let a = md_to_ast(
        &c,
        &AstOptions::default(),
        Some("packages/peisar_fs/README.md".to_string()),
    );
    let json = serde_json::to_string_pretty(&a).unwrap();
    std::fs::write("aa.json", json).ok();
}

// use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

// fn main() {
//     let code = r#"
// fn main() {
//     println!("Hello, World");
// }
//     "#;
//     let syntax_set = SyntaxSet::load_defaults_newlines();
//     let syntax_reference = syntax_set.find_syntax_by_token("rust").unwrap();
//     let theme = ThemeSet::load_defaults().themes["base16-ocean.dark"].clone();
//     let html = highlighted_html_for_string(code, &syntax_set, &syntax_reference, &theme).unwrap();
//     println!("{}", html);
// }
