mod options;
mod parsers;

mod tokens;
mod utils;
pub use options::AstOptions;
pub use parsers::md_to_ast;

// Parse a Markdown string into a [`Document`] using default options.
// fn parse(input: &str) -> Document {
//     parse_with(input, &ParseOptions::default())
// }

// Parse a Markdown string into a [`Document`] with the given options.
//
// Options control which extensions (GFM, Kramdown) are active.
// pub fn md_to_ast(input: &str, opts: &AstOptions) -> Document {
//     let line_starts = block::compute_line_starts(input);
//     let lines: Vec<&str> = input.lines().collect();
//     let mut p = block::ParserState::new(input, &lines, &line_starts, opts);
//     let start = p.position_at(0);

//     let mut blocks = Vec::new();

//     while !p.is_done() {
//         if p.skip_blank_lines() {
//             continue;
//         }
//         if let Some(block) = p.parse_block() {
//             blocks.push(block);
//         } else {
//             p.advance(); // safety net — never loop forever
//         }
//     }

//     let end = p.position_at(p.pos.min(p.lines.len()));

//     Document {
//         node_type: "root",
//         children: blocks,
//         position: Span::new(start, end),
//     }
// }
