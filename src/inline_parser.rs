use crate::ast::{Image, Inline, Link};
use crate::util::{escape_url_parens, pct_encode};
use std::collections::HashMap;

pub fn parse_inline(text: &str, link_aliases: &HashMap<String, String>) -> Vec<Inline> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        // [[link]] or [[link][description]]
        if i + 1 < len && chars[i] == '[' && chars[i + 1] == '[' {
            if let Some((node, consumed)) = parse_link_or_image(&chars, i, link_aliases) {
                result.push(node);
                i += consumed;
                continue;
            }
        }
        // Bare URL
        let remaining: String = chars[i..].iter().collect();
        if remaining.starts_with("http://") || remaining.starts_with("https://") {
            let url_start = i;
            while i < len && !chars[i].is_whitespace() {
                i += 1;
            }
            let url: String = chars[url_start..i].iter().collect();
            let url_escaped = escape_url_parens(&url);
            result.push(Inline::Link(Link {
                url: url_escaped,
                text: vec![Inline::Text(url.clone())],
            }));
            continue;
        }
        // *bold*
        if chars[i] == '*' {
            if let Some((inner, n)) = markup_at(&chars, i, '*') {
                let inner_nodes = parse_inline(&inner, link_aliases);
                result.push(Inline::Bold(inner_nodes));
                i += n;
                continue;
            }
        }
        // /italic/
        if chars[i] == '/' {
            if let Some((inner, n)) = markup_at(&chars, i, '/') {
                let inner_nodes = parse_inline(&inner, link_aliases);
                result.push(Inline::Italic(inner_nodes));
                i += n;
                continue;
            }
        }
        // +strikethrough+
        if chars[i] == '+' {
            if let Some((inner, n)) = markup_at(&chars, i, '+') {
                let inner_nodes = parse_inline(&inner, link_aliases);
                result.push(Inline::StrikeThrough(inner_nodes));
                i += n;
                continue;
            }
        }
        // =verbatim=
        if chars[i] == '=' {
            if let Some((inner, n)) = markup_at(&chars, i, '=') {
                result.push(Inline::Verbatim(inner));
                i += n;
                continue;
            }
        }
        // ~code~
        if chars[i] == '~' {
            if let Some((inner, n)) = markup_at(&chars, i, '~') {
                result.push(Inline::Code(inner));
                i += n;
                continue;
            } else {
                result.push(Inline::Text("~".to_string()));
                i += 1;
                continue;
            }
        }
        // Regular character
        result.push(Inline::Text(chars[i].to_string()));
        i += 1;
    }
    result
}

fn markup_at(chars: &[char], start: usize, delim: char) -> Option<(String, usize)> {
    let mut i = start + 1;
    while i < chars.len() {
        if chars[i] == delim && (i + 1 == chars.len() || !chars[i + 1].is_alphanumeric()) {
            let inner: String = chars[start + 1..i].iter().collect();
            let consumed = i - start + 1;
            return Some((inner, consumed));
        }
        i += 1;
    }
    None
}

fn parse_link_or_image(
    chars: &[char],
    start: usize,
    link_aliases: &HashMap<String, String>,
) -> Option<(Inline, usize)> {
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
    if desc.is_none() && i < chars.len() && chars[i] == ']' {
        i += 1;
    }
    let consumed = i - start;
    // Check for file: image or link
    if let Some(path) = target.strip_prefix("file:") {
        let lower = path.to_lowercase();
        let is_image = lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".gif")
            || lower.ends_with(".svg")
            || lower.ends_with(".webp");
        let encoded = pct_encode(path);
        if is_image {
            let alt = desc.as_deref().unwrap_or(path).to_string();
            return Some((
                Inline::Image(Image {
                    url: encoded,
                    alt_text: Some(alt),
                }),
                consumed,
            ));
        } else {
            let text = desc.as_deref().unwrap_or(path).to_string();
            let text_nodes = parse_inline(&text, link_aliases);
            return Some((
                Inline::Link(Link {
                    url: format!("file:{}", encoded),
                    text: text_nodes,
                }),
                consumed,
            ));
        }
    }
    // Check link aliases
    if let Some(url) = link_aliases.get(&target) {
        let text = desc.as_deref().unwrap_or(&target).to_string();
        let text_nodes = parse_inline(&text, link_aliases);
        let url_escaped = escape_url_parens(url);
        return Some((
            Inline::Link(Link {
                url: url_escaped,
                text: text_nodes,
            }),
            consumed,
        ));
    }
    let url = escape_url_parens(&target);
    if let Some(d) = desc {
        let text_nodes = parse_inline(&d, link_aliases);
        Some((
            Inline::Link(Link {
                url,
                text: text_nodes,
            }),
            consumed,
        ))
    } else {
        let text = target.replace('&', "\\&");
        Some((
            Inline::Link(Link {
                url,
                text: vec![Inline::Text(text)],
            }),
            consumed,
        ))
    }
}
