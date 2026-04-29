use crate::util::{escape_url_parens, markup_at, pct_encode};
use std::collections::HashMap;

pub fn convert_inline(text: &str, link_aliases: &HashMap<String, String>) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // [[...]] link
        if i + 1 < len && chars[i] == '[' && chars[i + 1] == '[' {
            if let Some((mdx, consumed)) = parse_link_at(&chars, i, link_aliases) {
                out.push_str(&mdx);
                i += consumed;
                continue;
            }
        }

        // Bare URL
        {
            let remaining: String = chars[i..].iter().collect();
            if remaining.starts_with("http://") || remaining.starts_with("https://") {
                let url_start = i;
                while i < len && !chars[i].is_whitespace() {
                    i += 1;
                }
                let url: String = chars[url_start..i].iter().collect();
                let url_escaped = escape_url_parens(&url);
                out.push_str(&format!("[{}]({})", url, url_escaped));
                continue;
            }
        }

        // *bold*
        if chars[i] == '*' {
            if let Some((inner, n)) = markup_at(&chars, i, '*') {
                out.push_str(&format!("**{}**", convert_inline(&inner, link_aliases)));
                i += n;
                continue;
            }
        }

        // /italic/
        if chars[i] == '/' {
            if let Some((inner, n)) = markup_at(&chars, i, '/') {
                out.push_str(&format!("*{}*", convert_inline(&inner, link_aliases)));
                i += n;
                continue;
            }
        }

        // +strikethrough+
        if chars[i] == '+' {
            if let Some((inner, n)) = markup_at(&chars, i, '+') {
                out.push_str(&format!("~~{}~~", convert_inline(&inner, link_aliases)));
                i += n;
                continue;
            }
        }

        // =verbatim=
        if chars[i] == '=' {
            if let Some((inner, n)) = markup_at(&chars, i, '=') {
                out.push_str(&format!("`{}`", inner));
                i += n;
                continue;
            }
        }

        // ~code~
        if chars[i] == '~' {
            if let Some((inner, n)) = markup_at(&chars, i, '~') {
                out.push_str(&format!("`{}`", inner));
                i += n;
                continue;
            } else {
                out.push_str("\\~");
                i += 1;
                continue;
            }
        }

        // Subscript
        if chars[i] == '_' && i > 0 && chars[i - 1].is_alphanumeric() {
            if i + 1 < len {
                let next = chars[i + 1];
                if next == '{' {
                    let mut j = i + 2;
                    while j < len && chars[j] != '}' {
                        out.push(chars[j]);
                        j += 1;
                    }
                    i = if j < len { j + 1 } else { j };
                    continue;
                }
            }
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}

fn parse_link_at(
    chars: &[char],
    start: usize,
    link_aliases: &HashMap<String, String>,
) -> Option<(String, usize)> {
    let mut i = start + 2;
    let mut target = String::new();
    let mut depth = 1;

    while i < chars.len() {
        match chars[i] {
            '[' => {
                depth += 1;
                target.push('[');
            }
            ']' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
                target.push(']');
            }
            c => target.push(c),
        }
        i += 1;
    }

    let desc = if i < chars.len() && chars[i] == '[' {
        i += 1;
        let mut d = String::new();
        while i < chars.len() {
            if chars[i] == ']' && i + 1 < chars.len() && chars[i + 1] == ']' {
                i += 2;
                break;
            }
            match chars[i] {
                '[' => d.push_str("\\["),
                ']' => d.push_str("\\]"),
                c => d.push(c),
            }
            i += 1;
        }
        Some(d)
    } else {
        None
    };

    if desc.is_none() {
        if i < chars.len() && chars[i] == ']' {
            i += 1;
        }
    }

    let consumed = i - start;

    if let Some(path) = target.strip_prefix("file:") {
        let lower = path.to_lowercase();
        let is_image = lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".gif")
            || lower.ends_with(".svg")
            || lower.ends_with(".webp");
        let encoded = pct_encode(path);
        if is_image && desc.is_none() {
            return Some((format!("[img]({})", encoded), consumed));
        } else {
            let display = desc.as_deref().unwrap_or(path).to_string();
            let display = convert_inline(&display, link_aliases);
            return Some((format!("[{}](file:{})", display, encoded), consumed));
        }
    }

    if let Some(url) = link_aliases.get(&target) {
        let display = desc.as_deref().unwrap_or(&target).to_string();
        let display = convert_inline(&display, link_aliases);
        let url = escape_url_parens(url);
        return Some((format!("[{}]({})", display, url), consumed));
    }

    let url = escape_url_parens(&target);
    if let Some(d) = desc {
        let display = convert_inline(&d, link_aliases);
        Some((format!("[{}]({})", display, url), consumed))
    } else {
        let display = target.replace('&', "\\&");
        Some((format!("[{}]({})", display, url), consumed))
    }
}
