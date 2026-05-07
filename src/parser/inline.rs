use crate::ast::Node;
use crate::util::escape_url_parens;
use std::collections::HashMap;

pub fn parse_inline(text: &str, link_aliases: &HashMap<String, String>) -> Vec<Node> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if i + 1 < len && chars[i] == '[' && chars[i + 1] == '[' {
            if let Some((node, consumed)) = parse_link_or_image(&chars, i, link_aliases) {
                result.push(node);
                i += consumed;
                continue;
            }
        }
        let remaining: String = chars[i..].iter().collect();
        if remaining.starts_with("http://") || remaining.starts_with("https://") {
            let url_start = i;
            while i < len && !chars[i].is_whitespace() {
                i += 1;
            }
            let url: String = chars[url_start..i].iter().collect();
            let url_escaped = escape_url_parens(&url);
            result.push(
                Node::new("link")
                    .with_children(vec![Node::text(&url)])
                    .data_str("url", &url_escaped),
            );
            continue;
        }
        if chars[i] == '*' {
            if let Some((inner, n)) = markup_at(&chars, i, '*') {
                let inner_nodes = parse_inline(&inner, link_aliases);
                result.push(Node::new("strong").with_children(inner_nodes));
                i += n;
                continue;
            }
        }
        if chars[i] == '/' {
            if let Some((inner, n)) = markup_at(&chars, i, '/') {
                let inner_nodes = parse_inline(&inner, link_aliases);
                result.push(Node::new("emphasis").with_children(inner_nodes));
                i += n;
                continue;
            }
        }
        if chars[i] == '+' {
            if let Some((inner, n)) = markup_at(&chars, i, '+') {
                let inner_nodes = parse_inline(&inner, link_aliases);
                result.push(Node::new("delete").with_children(inner_nodes));
                i += n;
                continue;
            }
        }
        if chars[i] == '=' {
            if let Some((inner, n)) = markup_at(&chars, i, '=') {
                result.push(Node::new("inlineCode").with_value(&inner));
                i += n;
                continue;
            }
        }
        if chars[i] == '~' {
            if let Some((inner, n)) = markup_at(&chars, i, '~') {
                result.push(Node::new("inlineCode").with_value(&inner));
                i += n;
                continue;
            } else {
                result.push(Node::text("~"));
                i += 1;
                continue;
            }
        }
        if chars[i] == '_' {
            if i + 1 < chars.len() && chars[i + 1] == '{' {
                if let Some((inner, n)) = parse_braced(&chars, i + 1) {
                    let inner_nodes = parse_inline(&inner, link_aliases);
                    result.push(Node::new("subscript").with_children(inner_nodes));
                    i += n + 1;
                    continue;
                }
            }
            if let Some((inner, n)) = markup_at(&chars, i, '_') {
                let inner_nodes = parse_inline(&inner, link_aliases);
                result.push(Node::new("underline").with_children(inner_nodes));
                i += n;
                continue;
            }
        }
        result.push(Node::text(&chars[i].to_string()));
        i += 1;
    }
    result
}

fn parse_braced(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut depth = 1;
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let inner: String = chars[start + 1..i].iter().collect();
                    return Some((inner, i - start + 1));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
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
) -> Option<(Node, usize)> {
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
    if let Some(path) = target.strip_prefix("file:") {
        let lower = path.to_lowercase();
        let is_image = lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".gif")
            || lower.ends_with(".svg")
            || lower.ends_with(".webp");
        let encoded = path.to_string();
        if is_image {
            let alt = desc.as_deref().unwrap_or(path).to_string();
            return Some((
                Node::new("image")
                    .data_str("url", &encoded)
                    .data_str("alt", &alt),
                consumed,
            ));
        } else {
            let text = desc.as_deref().unwrap_or(path).to_string();
            let text_nodes = parse_inline(&text, link_aliases);
            return Some((
                Node::new("link")
                    .with_children(text_nodes)
                    .data_str("url", &format!("file:{}", encoded)),
                consumed,
            ));
        }
    }
    if let Some(url) = link_aliases.get(&target) {
        let text = desc.as_deref().unwrap_or(&target).to_string();
        let text_nodes = parse_inline(&text, link_aliases);
        let url_escaped = escape_url_parens(url);
        return Some((
            Node::new("link")
                .with_children(text_nodes)
                .data_str("url", &url_escaped),
            consumed,
        ));
    }
    if let Some((alias, suffix)) = target.split_once(':') {
        if let Some(base_url) = link_aliases.get(alias) {
            let full_url = format!("{}{}", base_url, suffix);
            let link_text = format!("{}:{}", alias, suffix);
            let text = desc.as_deref().unwrap_or(&link_text).to_string();
            let text_nodes = parse_inline(&text, link_aliases);
            let url_escaped = escape_url_parens(&full_url);
            return Some((
                Node::new("link")
                    .with_children(text_nodes)
                    .data_str("url", &url_escaped),
                consumed,
            ));
        }
    }
    let url = escape_url_parens(&target);
    if let Some(d) = desc {
        let text_nodes = parse_inline(&d, link_aliases);
        Some((
            Node::new("link")
                .with_children(text_nodes)
                .data_str("url", &url),
            consumed,
        ))
    } else {
        let text = target.replace('&', "\\&");
        Some((
            Node::new("link")
                .with_children(vec![Node::text(&text)])
                .data_str("url", &url),
            consumed,
        ))
    }
}
