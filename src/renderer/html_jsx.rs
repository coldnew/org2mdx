/// Void (self-closing) HTML elements that require `/>` in JSX
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// HTML attribute name → canonical JSX attribute name (used when parsing HTML)
const ATTR_HTML_TO_CANONICAL: &[(&str, &str)] = &[
    ("class", "className"),
    ("for", "htmlFor"),
    ("tabindex", "tabIndex"),
    ("onclick", "onClick"),
    ("onchange", "onChange"),
    ("onsubmit", "onSubmit"),
    ("onfocus", "onFocus"),
    ("onblur", "onBlur"),
    ("onkeydown", "onKeyDown"),
    ("onkeyup", "onKeyUp"),
    ("onload", "onLoad"),
    ("onerror", "onError"),
    ("readonly", "readOnly"),
    ("maxlength", "maxLength"),
];

/// Canonical JSX attribute name → HTML attribute name (used when serializing to HTML)
const ATTR_CANONICAL_TO_HTML: &[(&str, &str)] = &[
    ("className", "class"),
    ("htmlFor", "for"),
    ("tabIndex", "tabindex"),
    ("onClick", "onclick"),
    ("onChange", "onchange"),
    ("onSubmit", "onsubmit"),
    ("onFocus", "onfocus"),
    ("onBlur", "onblur"),
    ("onKeyDown", "onkeydown"),
    ("onKeyUp", "onkeyup"),
    ("onLoad", "onload"),
    ("onError", "onerror"),
    ("readOnly", "readonly"),
    ("maxLength", "maxlength"),
];

// ------------------------------------------------------------
// Unified HTML/JSX fragment syntax tree
// ------------------------------------------------------------

#[derive(Debug, Clone)]
enum HtmlNode {
    Text(String),
    Comment(String),
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<HtmlNode>,
        void: bool,
    },
}

#[derive(Clone, Copy, PartialEq)]
enum ParseMode {
    Html,
    Jsx,
}

#[derive(Clone, Copy)]
enum SerializeMode {
    Html,
    Jsx,
}

// ------------------------------------------------------------
// Public API
// ------------------------------------------------------------

pub fn html_to_jsx(html: &str) -> String {
    let nodes = parse_nodes(html, ParseMode::Html);
    serialize_nodes(&nodes, SerializeMode::Jsx)
}

pub fn jsx_to_html(jsx: &str) -> String {
    let nodes = parse_nodes(jsx, ParseMode::Jsx);
    serialize_nodes(&nodes, SerializeMode::Html)
}

// ------------------------------------------------------------
// Parser — HTML/JSX string → unified HtmlNode tree
// ------------------------------------------------------------

fn parse_nodes(input: &str, mode: ParseMode) -> Vec<HtmlNode> {
    let mut nodes = Vec::new();
    let mut remaining = input;
    while let Some((node, rest)) = parse_single_node(remaining, mode) {
        nodes.push(node);
        remaining = rest;
    }
    nodes
}

fn parse_single_node(input: &str, mode: ParseMode) -> Option<(HtmlNode, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return None;
    }
    match mode {
        ParseMode::Html => {
            if input.starts_with("<!--") {
                if let Some(r) = parse_comment(input, mode) {
                    return Some(r);
                }
                return Some((HtmlNode::Text("<!--".to_string()), &input[4..]));
            }
            if input.starts_with('<') {
                let next = *input.as_bytes().get(1)?;
                if next.is_ascii_alphabetic() {
                    if let Some(r) = parse_element(input, mode) {
                        return Some(r);
                    }
                }
                if next == b'!' || next == b'?' {
                    if let Some(gt) = input.find('>') {
                        return Some((HtmlNode::Text(String::new()), &input[gt + 1..]));
                    }
                }
            }
        }
        ParseMode::Jsx => {
            if input.starts_with("{/*") {
                if let Some(r) = parse_comment(input, mode) {
                    return Some(r);
                }
                return Some((HtmlNode::Text("{/*".to_string()), &input[3..]));
            }
            if input.starts_with('<') {
                let next = *input.as_bytes().get(1)?;
                if next.is_ascii_alphabetic() {
                    if let Some(r) = parse_element(input, mode) {
                        return Some(r);
                    }
                }
                if next == b'!' || next == b'?' {
                    if let Some(gt) = input.find('>') {
                        return Some((HtmlNode::Text(String::new()), &input[gt + 1..]));
                    }
                }
            }
        }
    }
    let (text, rest) = parse_text(input, mode);
    Some((text, rest))
}

fn parse_comment(input: &str, mode: ParseMode) -> Option<(HtmlNode, &str)> {
    match mode {
        ParseMode::Html => {
            let after = &input[4..];
            let end = after.find("-->")?;
            let text = after[..end].trim().to_string();
            Some((HtmlNode::Comment(text), &after[end + 3..]))
        }
        ParseMode::Jsx => {
            let after = &input[3..];
            let end = after.find("*/}")?;
            let text = after[..end].trim().to_string();
            Some((HtmlNode::Comment(text), &after[end + 3..]))
        }
    }
}

fn parse_element(input: &str, mode: ParseMode) -> Option<(HtmlNode, &str)> {
    let rest = &input[1..];

    let tag_end = rest
        .find(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/')
        .unwrap_or(rest.len());
    let tag_name = rest[..tag_end].to_lowercase();
    let mut rest = &rest[tag_end..];

    let is_void = VOID_ELEMENTS.contains(&tag_name.as_str());
    let mut attrs = Vec::new();

    loop {
        rest = rest.trim_start();
        if rest.is_empty() || rest.starts_with('>') || rest.starts_with("/>") {
            break;
        }
        let name_end = rest
            .find(|c: char| c.is_ascii_whitespace() || c == '=' || c == '>' || c == '/')
            .unwrap_or(rest.len());
        if name_end == 0 {
            break;
        }
        let name = &rest[..name_end];
        rest = &rest[name_end..];
        rest = rest.trim_start();

        if !rest.starts_with('=') {
            let cname = normalize_attr_name(name, mode);
            attrs.push((cname, String::new()));
            continue;
        }
        rest = &rest[1..];
        rest = rest.trim_start();

        if rest.is_empty() {
            break;
        }
        let bytes = rest.as_bytes();
        let value: String;
        if bytes[0] == b'"' || bytes[0] == b'\'' {
            let quote = bytes[0] as char;
            if let Some(end) = rest[1..].find(quote) {
                value = rest[1..end + 1].to_string();
                rest = &rest[end + 2..];
            } else {
                break;
            }
        } else if mode == ParseMode::Jsx && bytes[0] == b'{' {
            let mut depth = 1;
            let mut pos = 1;
            while pos < rest.len() && depth > 0 {
                match rest.as_bytes()[pos] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                pos += 1;
            }
            if depth == 0 {
                value = rest[..pos].to_string();
                rest = &rest[pos..];
            } else {
                break;
            }
        } else {
            let end = rest
                .find(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/')
                .unwrap_or(rest.len());
            value = rest[..end].to_string();
            rest = &rest[end..];
        }
        let cname = normalize_attr_name(name, mode);
        attrs.push((cname, value));
    }

    rest = rest.trim_start();
    if rest.starts_with("/>") {
        return Some((
            HtmlNode::Element {
                tag: tag_name,
                attrs,
                children: vec![],
                void: true,
            },
            &rest[2..],
        ));
    }
    if rest.starts_with('>') {
        rest = &rest[1..];
        if is_void {
            return Some((
                HtmlNode::Element {
                    tag: tag_name,
                    attrs,
                    children: vec![],
                    void: true,
                },
                rest,
            ));
        }
        let (children, rest) = parse_children(rest, &tag_name, mode);
        return Some((
            HtmlNode::Element {
                tag: tag_name,
                attrs,
                children,
                void: false,
            },
            rest,
        ));
    }
    None
}

fn parse_children<'a>(
    input: &'a str,
    parent_tag: &str,
    mode: ParseMode,
) -> (Vec<HtmlNode>, &'a str) {
    let mut children = Vec::new();
    let mut remaining = input;
    let close_tag = format!("</{}>", parent_tag);
    loop {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }
        if remaining.starts_with(&close_tag) {
            return (children, &remaining[close_tag.len()..]);
        }
        if remaining.starts_with("</") {
            break;
        }
        if let Some((node, rest)) = parse_single_node(remaining, mode) {
            children.push(node);
            remaining = rest;
        } else {
            break;
        }
    }
    (children, remaining)
}

fn parse_text(input: &str, mode: ParseMode) -> (HtmlNode, &str) {
    let bytes = input.as_bytes();
    let mut end = 0;
    while end < bytes.len() {
        if bytes[end] == b'<' {
            if end + 1 < bytes.len() {
                let next = bytes[end + 1];
                if next.is_ascii_alphabetic() || next == b'/' || next == b'!' || next == b'?' {
                    break;
                }
            }
        }
        if mode == ParseMode::Jsx && bytes[end] == b'{' {
            if input[end..].starts_with("{/*") {
                break;
            }
        }
        end += 1;
    }
    if end == 0 {
        (HtmlNode::Text(String::new()), input)
    } else {
        (HtmlNode::Text(input[..end].to_string()), &input[end..])
    }
}

// ------------------------------------------------------------
// Serializer — unified HtmlNode tree → HTML or JSX string
// ------------------------------------------------------------

fn serialize_nodes(nodes: &[HtmlNode], mode: SerializeMode) -> String {
    let mut out = String::new();
    for n in nodes {
        out.push_str(&serialize_node(n, mode));
    }
    out
}

fn serialize_node(node: &HtmlNode, mode: SerializeMode) -> String {
    match node {
        HtmlNode::Text(t) => t.clone(),
        HtmlNode::Comment(t) => match mode {
            SerializeMode::Html => format!("<!-- {} -->", t),
            SerializeMode::Jsx => format!("{{/* {} */}}", t),
        },
        HtmlNode::Element {
            tag,
            attrs,
            children,
            void,
        } => {
            let a = serialize_attrs(attrs, mode);
            match mode {
                SerializeMode::Html => {
                    if *void {
                        format!("<{}{}>", tag, a)
                    } else {
                        format!(
                            "<{}{}>{}</{}>",
                            tag,
                            a,
                            serialize_nodes(children, mode),
                            tag
                        )
                    }
                }
                SerializeMode::Jsx => {
                    if *void {
                        format!("<{}{} />", tag, a)
                    } else {
                        format!(
                            "<{}{}>{}</{}>",
                            tag,
                            a,
                            serialize_nodes(children, mode),
                            tag
                        )
                    }
                }
            }
        }
    }
}

fn serialize_attrs(attrs: &[(String, String)], mode: SerializeMode) -> String {
    let mut out = String::new();
    for (name, val) in attrs {
        let (dname, dval) = match mode {
            SerializeMode::Html => {
                let hname = canonical_to_html_attr(name);
                if name == "style" {
                    (hname, convert_style_to_html(val))
                } else {
                    (hname, val.clone())
                }
            }
            SerializeMode::Jsx => {
                if name == "style" {
                    ("style".to_string(), convert_style_to_jsx(val))
                } else {
                    (name.clone(), val.clone())
                }
            }
        };
        if dval.is_empty() {
            out.push_str(&format!(" {}", dname));
        } else {
            out.push_str(&format!(" {}=\"{}\"", dname, dval));
        }
    }
    out
}

// ------------------------------------------------------------
// Helpers — attribute name normalisation & style conversion
// ------------------------------------------------------------

fn normalize_attr_name(name: &str, mode: ParseMode) -> String {
    match mode {
        ParseMode::Html => {
            let lower = name.to_lowercase();
            for (from, to) in ATTR_HTML_TO_CANONICAL {
                if from == &lower {
                    return to.to_string();
                }
            }
            name.to_string()
        }
        ParseMode::Jsx => name.to_string(),
    }
}

fn canonical_to_html_attr(name: &str) -> String {
    for (from, to) in ATTR_CANONICAL_TO_HTML {
        if name == *from {
            return to.to_string();
        }
    }
    name.to_string()
}

fn convert_style_to_jsx(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let props: Vec<String> = value
        .split(';')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .filter_map(|p| {
            let mut kv = p.splitn(2, ':');
            let key = kv.next()?.trim();
            let val = kv.next()?.trim();
            Some(format!("{}: '{}'", kebab_to_camel(key), val))
        })
        .collect();
    if props.is_empty() {
        String::new()
    } else {
        format!("{{{{{}}}}}", props.join(", "))
    }
}

fn convert_style_to_html(value: &str) -> String {
    let inner = value.trim();
    if inner.starts_with("{{") && inner.ends_with("}}") {
        let obj = &inner[2..inner.len() - 2];
        let props: Vec<String> = obj
            .split(',')
            .filter_map(|p| {
                let mut kv = p.splitn(2, ':');
                let key = kv.next()?.trim();
                let val = kv.next()?.trim().trim_matches('\'').trim_matches('"');
                Some(format!("{}: {}", camel_to_kebab(key), val))
            })
            .collect();
        props.join("; ")
    } else {
        inner.to_string()
    }
}

fn kebab_to_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper = false;
    for c in s.chars() {
        if c == '-' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn camel_to_kebab(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}
