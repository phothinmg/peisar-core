use crate::options::AstOptions;
use crate::tokens::{
    span::{Position, Span},
    token::{EmphasisLevel, Inline},
};

/// Parse inline markdown into a list of [`Inline`] nodes.
/// Parse inline markdown with the given options (controls GFM features).
pub fn parse_inline(input: &str, options: Option<&AstOptions>) -> Vec<Inline> {
    let binding = AstOptions::default();
    let opts = options.unwrap_or(&binding);
    let mut tokens: Vec<Inline> = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let ctx = InlineCtx::new(input, &chars);
    let mut i = 0;
    let mut text = String::new();
    let mut text_start = 0usize;

    while i < chars.len() {
        let c = chars[i];

        // Inline code: `code` or ``code``
        if c == '`' {
            let tick_count = chars[i..].iter().take_while(|&&ch| ch == '`').count();
            if let Some((code, end)) = match_inline_code(&chars, i, tick_count) {
                flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
                tokens.push(Inline::Code {
                    code,
                    pos: Span::new(ctx.position(i, chars.len()), ctx.position(end, chars.len())),
                });
                i = end;
                text_start = i;
                continue;
            }
        }

        // GFM strikethrough: ~~text~~
        if opts.gfm && c == '~' && i + 1 < chars.len() && chars[i + 1] == '~' {
            if let Some((node, end)) = match_strikethrough(&chars, i, &ctx, options) {
                flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
                tokens.push(node);
                i = end;
                text_start = i;
                continue;
            }
        }

        // Image: ![alt](url)
        if c == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
            if let Some((img, end)) = match_image(&chars, i, &ctx) {
                flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
                tokens.push(img);
                i = end;
                text_start = i;
                continue;
            }
        }

        // Link: [text](url)
        if c == '[' {
            if let Some((link, end)) = match_link(&chars, i, &ctx, options) {
                flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
                tokens.push(link);
                i = end;
                text_start = i;
                continue;
            }
        }

        // GFM autolink: bare URLs
        if opts.gfm && (c == 'h' || c == 'w') {
            if let Some((url, end)) = match_autolink(&chars, i) {
                flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
                tokens.push(Inline::Link {
                    text: vec![Inline::Text {
                        value: url.clone(),
                        pos: Span::new(
                            ctx.position(i, chars.len()),
                            ctx.position(end, chars.len()),
                        ),
                    }],
                    url,
                    title: None,
                    autolink: true,
                    pos: Span::new(ctx.position(i, chars.len()), ctx.position(end, chars.len())),
                });
                i = end;
                text_start = i;
                continue;
            }
        }

        // Emphasis: **bold** / *italic* / __bold__ / _italic_
        if c == '*' || c == '_' {
            if let Some((node, end)) = match_emphasis(&chars, i, c, &ctx, options) {
                flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
                tokens.push(node);
                i = end;
                text_start = i;
                continue;
            }
        }

        // Hard break: two trailing spaces + newline, or backslash + newline
        if c == '\n' {
            if text.ends_with("  ") {
                text.truncate(text.len() - 2);
                flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
                tokens.push(Inline::HardBreak {
                    pos: Span::new(
                        ctx.position(i, chars.len()),
                        ctx.position(i + 1, chars.len()),
                    ),
                });
                i += 1;
                text_start = i;
                continue;
            }
            if text.ends_with('\\') {
                text.truncate(text.len() - 1);
                flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
                tokens.push(Inline::HardBreak {
                    pos: Span::new(
                        ctx.position(i, chars.len()),
                        ctx.position(i + 1, chars.len()),
                    ),
                });
                i += 1;
                text_start = i;
                continue;
            }
            flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
            tokens.push(Inline::SoftBreak {
                pos: Span::new(
                    ctx.position(i, chars.len()),
                    ctx.position(i + 1, chars.len()),
                ),
            });
            i += 1;
            text_start = i;
            continue;
        }

        // Inline HTML (very naive: <tag ...> or </tag>)
        if c == '<' {
            if let Some(end) = match_inline_html(&chars, i) {
                flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
                let html: String = chars[i..end].iter().collect();
                tokens.push(Inline::HtmlInline {
                    html,
                    pos: Span::new(ctx.position(i, chars.len()), ctx.position(end, chars.len())),
                });
                i = end;
                text_start = i;
                continue;
            }
        }

        text.push(c);
        i += 1;
    }

    flush_text(&mut tokens, &mut text, &mut text_start, &ctx, &chars);
    tokens
}

pub struct InlineCtx {
    /// `line_starts_char[l]` = char index at which line `l` begins.
    line_starts_char: Vec<usize>,
    /// `char_byte_offsets[i]` = byte offset of `chars[i]`.
    char_byte_offsets: Vec<usize>,
}

impl InlineCtx {
    fn new(input: &str, chars: &[char]) -> Self {
        let mut line_starts_char = vec![0usize];
        let mut char_byte_offsets = Vec::with_capacity(chars.len());
        let mut byte = 0usize;
        for c in chars {
            char_byte_offsets.push(byte);
            if *c == '\n' {
                line_starts_char.push(char_byte_offsets.len());
            }
            byte += c.len_utf8();
        }
        let _ = input;
        Self {
            line_starts_char,
            char_byte_offsets,
        }
    }

    /// Position at the given char index.
    pub fn position(&self, i: usize, chars_len: usize) -> Position {
        let i = i.min(chars_len);
        let line = match self.line_starts_char.binary_search(&i) {
            Ok(l) => l,
            Err(l) => l.saturating_sub(1),
        };
        let line_start = self.line_starts_char[line];
        let column = i - line_start;
        let offset = if i < self.char_byte_offsets.len() {
            self.char_byte_offsets[i]
        } else {
            self.char_byte_offsets.last().copied().unwrap_or(0)
                + if chars_len > 0 && self.char_byte_offsets.len() > 1 {
                    self.char_byte_offsets[self.char_byte_offsets.len() - 1]
                        - self.char_byte_offsets[self.char_byte_offsets.len() - 2]
                } else {
                    0
                }
        };
        Position {
            line,
            column,
            offset,
        }
    }
}
// ---------------------------------------------------------------------------
// GFM - Inline Parsers
// ---------------------------------------------------------------------------
/// Match GFM strikethrough `~~text~~`. Returns `(children, end_index)`.
fn match_strikethrough(
    chars: &[char],
    start: usize,
    ctx: &InlineCtx,
    options: Option<&AstOptions>,
) -> Option<(Inline, usize)> {
    if chars[start] != '~' || start + 1 >= chars.len() || chars[start + 1] != '~' {
        return None;
    }

    let after = &chars[start + 2..];
    let after_str: String = after.iter().collect();
    let close = after_str.find("~~")?;
    let inner: String = after_str[..close].to_string();
    let children = parse_inline(&inner, options);
    let end = start + 2 + close + 2;

    Some((
        Inline::Strikethrough {
            children,
            pos: Span::new(
                ctx.position(start, chars.len()),
                ctx.position(end, chars.len()),
            ),
        },
        end,
    ))
}
fn find_url_end(chars: &[char], from: usize) -> usize {
    let mut i = from;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() || c == '<' || c == '>' {
            break;
        }
        // Trailing punctuation: . , ; : ! ? ) — but not if part of URL path
        if (c == '.' || c == ',' || c == ';' || c == ':' || c == '!' || c == '?' || c == ')')
            && (i + 1 >= chars.len() || chars[i + 1].is_whitespace())
        {
            break;
        }
        i += 1;
    }
    i
}
/// Detect a bare URL for GFM autolink (www.example.com or https://example.com).
/// Returns `(url, end_index)` relative to `chars`.
fn match_autolink(chars: &[char], start: usize) -> Option<(String, usize)> {
    // Check for https:// or http://
    let protocols: [&[char]; 2] = [
        &['h', 't', 't', 'p', 's', ':', '/', '/'],
        &['h', 't', 't', 'p', ':', '/', '/'],
    ];

    for proto in protocols {
        if start + proto.len() <= chars.len() && chars[start..start + proto.len()] == *proto {
            let end = find_url_end(chars, start + proto.len());
            let url: String = chars[start..end].iter().collect();
            return Some((url, end));
        }
    }

    // Check for www.
    let www: [char; 4] = ['w', 'w', 'w', '.'];
    if start + www.len() <= chars.len() && &chars[start..start + www.len()] == &www {
        let end = find_url_end(chars, start + www.len());
        let url: String = chars[start..end].iter().collect();
        // Prepend https:// for www.
        return Some((format!("https://{url}"), end));
    }

    None
}

/// get text node
fn flush_text(
    tokens: &mut Vec<Inline>,
    text: &mut String,
    text_start: &mut usize,
    ctx: &InlineCtx,
    chars: &[char],
) {
    if !text.is_empty() {
        let start = *text_start;
        let end = start + text.chars().count();
        let pos = Span::new(
            ctx.position(start, chars.len()),
            ctx.position(end, chars.len()),
        );
        let value = std::mem::take(text);
        tokens.push(Inline::Text { value, pos })
    }
}

fn match_inline_code(chars: &[char], start: usize, tick_count: usize) -> Option<(String, usize)> {
    let open: String = std::iter::repeat('`').take(tick_count).collect();
    let rest = &chars[start + tick_count..];
    let rest_str: String = rest.iter().collect();
    let close_idx = rest_str.find(&open)?;
    let code = rest_str[..close_idx].to_string();
    let end = start + tick_count + close_idx + tick_count;
    Some((code, end))
}

fn match_link(
    chars: &[char],
    start: usize,
    ctx: &InlineCtx,
    options: Option<&AstOptions>,
) -> Option<(Inline, usize)> {
    let (text, after_text) = match_bracket(chars, start, '[')?;
    let (url, title, after_url) = match_paren(chars, after_text)?;
    let children = parse_inline(&text, options);
    Some((
        Inline::Link {
            text: children,
            url,
            title,
            autolink: false,
            pos: Span::new(
                ctx.position(start, chars.len()),
                ctx.position(after_url, chars.len()),
            ),
        },
        after_url,
    ))
}

fn match_image(chars: &[char], start: usize, ctx: &InlineCtx) -> Option<(Inline, usize)> {
    let (alt, after_text) = match_bracket(chars, start + 1, '[')?;
    let (url, title, after_url) = match_paren(chars, after_text)?;
    Some((
        Inline::Image {
            alt,
            url,
            title,
            pos: Span::new(
                ctx.position(start, chars.len()),
                ctx.position(after_url, chars.len()),
            ),
        },
        after_url,
    ))
}

/// Match `[...content...]` starting at `chars[start]`.
fn match_bracket(chars: &[char], start: usize, open: char) -> Option<(String, usize)> {
    if chars.get(start)? != &open {
        return None;
    }
    let close = if open == '[' { ']' } else { ')' };
    let mut depth = 1;
    let mut i = start + 1;
    let mut content = String::new();
    while i < chars.len() {
        let c = chars[i];
        if c == open {
            depth += 1;
            content.push(c);
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return Some((content, i + 1));
            }
            content.push(c);
        } else {
            content.push(c);
        }
        i += 1;
    }
    None
}

/// Match `(...)` starting at `chars[start]`.
fn match_paren(chars: &[char], start: usize) -> Option<(String, Option<String>, usize)> {
    if chars.get(start)? != &'(' {
        return None;
    }
    let mut depth = 1;
    let mut i = start + 1;
    let mut raw = String::new();
    while i < chars.len() {
        let c = chars[i];
        if c == '(' {
            depth += 1;
            raw.push(c);
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                let (url, title) = parse_link_dest(&raw);
                return Some((url, title, i + 1));
            }
            raw.push(c);
        } else {
            raw.push(c);
        }
        i += 1;
    }
    None
}

/// Split `url  "title"` into components.
fn parse_link_dest(raw: &str) -> (String, Option<String>) {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_suffix('"') {
        if let Some(qidx) = rest.rfind('"') {
            let url = rest[..qidx].trim().to_string();
            let title = rest[qidx + 1..].trim().to_string();
            return (url, Some(title));
        }
    }
    if let Some(rest) = trimmed.strip_suffix('\'') {
        if let Some(qidx) = rest.rfind('\'') {
            let url = rest[..qidx].trim().to_string();
            let title = rest[qidx + 1..].trim().to_string();
            return (url, Some(title));
        }
    }
    (trimmed.to_string(), None)
}

fn match_emphasis(
    chars: &[char],
    start: usize,
    marker: char,
    ctx: &InlineCtx,
    options: Option<&AstOptions>,
) -> Option<(Inline, usize)> {
    let run = chars[start..].iter().take_while(|&c| *c == marker).count();
    // Try bold first (double marker)
    if run >= 2 {
        let close = find_emphasis_close(chars, start + 2, marker, 2);
        if let Some(end) = close {
            let inner: String = chars[start + 2..end].iter().collect();
            let children = parse_inline(&inner, options);
            let after = end + 2;
            return Some((
                Inline::Emphasis {
                    level: EmphasisLevel::Bold,
                    children,
                    pos: Span::new(
                        ctx.position(start, chars.len()),
                        ctx.position(after, chars.len()),
                    ),
                },
                after,
            ));
        }
    }
    // Single — italic
    let end = find_emphasis_close(chars, start + 1, marker, 1)?;
    let inner: String = chars[start + 1..end].iter().collect();
    let children = parse_inline(&inner, options);
    let after = end + 1;
    Some((
        Inline::Emphasis {
            level: EmphasisLevel::Italic,
            children,
            pos: Span::new(
                ctx.position(start, chars.len()),
                ctx.position(after, chars.len()),
            ),
        },
        after,
    ))
}

fn find_emphasis_close(chars: &[char], from: usize, marker: char, count: usize) -> Option<usize> {
    let mut i = from;
    while i < chars.len() {
        if chars[i] == marker {
            let run = chars[i..].iter().take_while(|&&c| c == marker).count();
            if run >= count {
                return Some(i);
            }
            i += run;
            continue;
        }
        i += 1;
    }
    None
}

fn match_inline_html(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start)? != &'<' {
        return None;
    }
    let mut i = start + 1;
    // comment
    if chars.get(i)? == &'!' {
        let comment_end = chars[start..]
            .windows(3)
            .position(|w| w == ['-', '-', '>'])?;
        return Some(start + comment_end + 3);
    }
    // closing tag
    if chars.get(i)? == &'/' {
        i += 1;
    }
    // tag name
    if chars.get(i)?.is_ascii_alphabetic() {
        while i < chars.len() && chars[i] != '>' {
            i += 1;
        }
        if i < chars.len() && chars[i] == '>' {
            return Some(i + 1);
        }
    }
    None
}
