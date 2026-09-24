use crate::tokens::Attributes;

/// Parse a Kramdown attribute block starting at the opening `{`.
/// Returns `(attrs, chars_consumed)` or `None` if not a valid block.
pub fn parse_attrs(s: &str) -> Option<(Attributes, usize)> {
    let s = s.trim_start();
    if !s.starts_with('{') {
        return None;
    }

    // Find the matching closing `}`.
    let chars: Vec<char> = s.chars().collect();
    let mut depth = 0i32;
    let mut end = 0usize;
    let mut in_string: Option<char> = None;
    for (i, &c) in chars.iter().enumerate() {
        match in_string {
            Some(q) => {
                if c == q {
                    in_string = None;
                }
            }
            None => {
                if c == '"' || c == '\'' {
                    in_string = Some(c);
                } else if c == '{' {
                    depth += 1;
                } else if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
            }
        }
    }

    if end == 0 {
        return None;
    }

    // Kramdown block attribute syntax is `{:...}` — skip the optional `:`
    // immediately after the opening `{`.
    let inner_start = if chars.get(1) == Some(&':') { 2 } else { 1 };
    let inner: String = chars[inner_start..end - 1].iter().collect();
    let attrs = parse_attr_tokens(&inner);
    Some((attrs, end))
}

/// Parse the tokens inside `{...}`.
fn parse_attr_tokens(inner: &str) -> Attributes {
    let mut attrs = Attributes::default();
    let chars: Vec<char> = inner.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        match chars[i] {
            // ID: #id
            '#' => {
                i += 1;
                let start = i;
                while i < chars.len() && is_ident_char(chars[i]) {
                    i += 1;
                }
                if i > start {
                    attrs.id = Some(chars[start..i].iter().collect());
                }
            }
            // Class: .class
            '.' => {
                i += 1;
                let start = i;
                while i < chars.len() && is_ident_char(chars[i]) {
                    i += 1;
                }
                if i > start {
                    attrs
                        .classes
                        .clone()
                        .unwrap()
                        .push(chars[start..i].iter().collect());
                }
            }
            // key="value" or key='value'
            c if is_ident_start(c) => {
                let key_start = i;
                while i < chars.len() && is_ident_char(chars[i]) {
                    i += 1;
                }
                let key: String = chars[key_start..i].iter().collect();

                // skip whitespace
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }

                if i < chars.len() && chars[i] == '=' {
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                        let q = chars[i];
                        i += 1;
                        let val_start = i;
                        while i < chars.len() && chars[i] != q {
                            i += 1;
                        }
                        let val: String = chars[val_start..i].iter().collect();
                        if i < chars.len() {
                            i += 1; // skip closing quote
                        }
                        attrs.attributes.clone().unwrap().push((key, val));
                    } else {
                        // unquoted value
                        let val_start = i;
                        while i < chars.len() && !chars[i].is_whitespace() {
                            i += 1;
                        }
                        let val: String = chars[val_start..i].iter().collect();
                        if !val.is_empty() {
                            attrs.attributes.clone().unwrap().push((key, val));
                        }
                    }
                } else {
                    // bare key → boolean attribute
                    attrs.attributes.clone().unwrap().push((key, String::new()));
                }
            }
            _ => {
                i += 1; // skip unknown char
            }
        }
    }

    attrs
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '-'
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}
