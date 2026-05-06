use crate::ast::Node;
use std::collections::HashSet;

const ORDERED_FRONTMATTER_KEYS: [&str; 10] = [
    "title", "date", "updated", "abbrlink", "author", "email", "options", "tags", "language",
    "alias",
];

pub fn render_org(root: &Node) -> String {
    let mut out = String::new();
    let org_opts = root.data.get("org").and_then(|org| org.get("options"));
    if !root.data.is_empty() {
        for key in ORDERED_FRONTMATTER_KEYS.iter() {
            if *key == "options" {
                if let Some(opts) = org_opts.and_then(|o| o.as_object()) {
                    let mut parts = Vec::new();
                    for (k, v) in opts {
                        let org_k = if k == "superscript" { "^" } else { k.as_str() };
                        let v_str = match v {
                            serde_json::Value::Bool(false) => "nil",
                            serde_json::Value::Bool(true) => "t",
                            serde_json::Value::String(s) => s.as_str(),
                            _ => continue,
                        };
                        parts.push(format!("{}:{}", org_k, v_str));
                    }
                    if !parts.is_empty() {
                        out.push_str(&format!("#+OPTIONS: {}\n", parts.join(" ")));
                    }
                }
                continue;
            }
            if let Some(value) = root.data.get(*key) {
                render_frontmatter_org(&mut out, key, value);
            }
        }
        let mut remaining_keys: Vec<&String> = root
            .data
            .keys()
            .filter(|k| {
                let k_lower = k.to_lowercase();
                !ORDERED_FRONTMATTER_KEYS.contains(&k_lower.as_str())
            })
            .collect();
        remaining_keys.sort();
        for key in remaining_keys {
            let value = &root.data[key];
            render_frontmatter_org(&mut out, key, value);
        }
        out.push('\n');
    }
    let link_abbreviations = collect_link_abbreviations(root);
    if !link_abbreviations.is_empty() {
        for (abbrev, base_url) in &link_abbreviations {
            out.push_str(&format!("#+LINK: {:6}{}\n", abbrev, base_url));
        }
        out.push('\n');
    }
    if let Some(children) = &root.children {
        for block in children {
            render_node_org(&mut out, block, &link_abbreviations);
        }
    }
    while out.ends_with('\n') {
        out.pop();
    }
    out.push('\n');
    out
}

fn render_frontmatter_org(out: &mut String, key: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::String(s) => out.push_str(&format!("#+{}: {}\n", key.to_uppercase(), s)),
        serde_json::Value::Array(list) => {
            if key.eq_ignore_ascii_case("tags") {
                let joined = list
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ");
                out.push_str(&format!("#+{}: {}\n", key.to_uppercase(), joined));
            } else {
                for item in list {
                    if let Some(s) = item.as_str() {
                        out.push_str(&format!("#+{}: {}\n", key.to_uppercase(), s));
                    }
                }
            }
        }
        _ => {}
    }
}

fn collect_link_abbreviations(root: &Node) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    collect_abbreviations_recursive(root, &mut result, &mut seen);
    result
}

fn collect_abbreviations_recursive(
    node: &Node,
    result: &mut Vec<(String, String)>,
    seen: &mut HashSet<String>,
) {
    if node.r#type == "link" {
        let text = render_inlines_simple(&node.children);
        if let Some((abbrev, path)) = text.split_once(':') {
            if !abbrev.is_empty() && !path.is_empty() && !seen.contains(abbrev) {
                let url = node.get_data_str("url").unwrap_or("");
                if url.ends_with(path) {
                    let base_url = &url[..url.len() - path.len()];
                    seen.insert(abbrev.to_string());
                    result.push((abbrev.to_string(), base_url.to_string()));
                }
            }
        }
    }
    if let Some(children) = &node.children {
        for child in children {
            collect_abbreviations_recursive(child, result, seen);
        }
    }
}

fn render_inlines_simple(children: &Option<Vec<Node>>) -> String {
    let mut out = String::new();
    if let Some(inlines) = children {
        for inline in inlines {
            if inline.r#type == "text" {
                if let Some(val) = &inline.value {
                    out.push_str(val);
                }
            }
        }
    }
    out
}

fn render_node_org(
    out: &mut String,
    node: &Node,
    link_abbreviations: &[(String, String)],
) {
    match node.r#type.as_str() {
        "heading" => {
            let depth: u64 = node
                .data
                .get("depth")
                .and_then(|v| match v {
                    &serde_json::Value::Number(ref n) => n.as_u64(),
                    _ => None,
                })
                .unwrap_or(1);
            let prefix = "*".repeat(depth as usize);
            let content = render_inlines_org(&node.children, link_abbreviations);
            out.push_str(&format!("{} {}\n\n", prefix, content));
            if let Some(tags) = node.get_data_list("tags") {
                if !tags.is_empty() {
                    let tag_strs: Vec<&str> = tags.iter().filter_map(|v| v.as_str()).collect();
                    out.push_str(&format!(" :{}:\n", tag_strs.join(":")));
                }
            }
        }
        "paragraph" => {
            let content = render_inlines_org(&node.children, link_abbreviations);
            if !content.is_empty() {
                if node.get_data_bool("hardLineBreak").unwrap_or(false) {
                    out.push_str(&format!("{}\\\\\n\n", content));
                } else {
                    out.push_str(&format!("{}\n\n", content));
                }
            }
        }
        "list" => render_list_org(out, node, 0, link_abbreviations),
        "code" => {
            let val = node.value.as_deref().unwrap_or("");
            let has_lang = node.get_data_str("lang").is_some();
            let single_line = !val.contains('\n');
            if !has_lang && single_line {
                for line in val.lines() {
                    out.push_str(&format!(": {}\n", line));
                }
                out.push('\n');
            } else {
                if has_lang {
                    if let Some(lang) = node.get_data_str("lang") {
                        out.push_str(&format!("#+BEGIN_SRC {}\n", lang));
                    }
                } else {
                    out.push_str("#+BEGIN_EXAMPLE\n");
                }
                let is_example = node
                    .get_data_str("block_type")
                    .map_or(false, |t| t == "example");
                for line in val.lines() {
                    if is_example {
                        out.push_str(&format!("{}\n", line));
                    } else {
                        out.push_str(&format!("  {}\n", line));
                    }
                }
                if val.ends_with('\n') {
                    out.pop();
                    out.push('\n');
                }
                if node.get_data_str("lang").is_some() {
                    out.push_str("#+END_SRC\n\n");
                } else {
                    out.push_str("#+END_EXAMPLE\n\n");
                }
            }
        }
        "blockquote" => {
            out.push_str("#+begin_quote\n");
            if let Some(children) = &node.children {
                for child in children {
                    render_node_org(out, child, link_abbreviations);
                }
            }
            out.push_str("#+end_quote\n\n");
        }
        "thematicBreak" => out.push_str("----\n\n"),
        "blankLine" => out.push('\n'),
        "html" => {
            if let Some(val) = &node.value {
                out.push_str(&format!("#+JSX: {}\n\n", val));
            }
        }
        "export" => {
            let export_type = node.get_data_str("lang").unwrap_or("");
            out.push_str(&format!("#+begin_export {}\n", export_type));
            if let Some(val) = &node.value {
                for line in val.lines() {
                    out.push_str(&format!("{}\n", line));
                }
            }
            out.push_str("#+end_export\n\n");
        }
        _ => {}
    }
}

fn render_list_org(
    out: &mut String,
    node: &Node,
    indent: usize,
    link_abbreviations: &[(String, String)],
) {
    let indent_str = "  ".repeat(indent);
    let ordered = node.get_data_bool("ordered").unwrap_or(false);
    if let Some(items) = &node.children {
        for (i, item) in items.iter().enumerate() {
            if ordered {
                out.push_str(&format!("{}{}. ", indent_str, i + 1));
            } else {
                out.push_str(&format!("{}- ", indent_str));
            }
            let mut item_out = String::new();
            if let Some(children) = &item.children {
                for child in children {
                    render_node_org(&mut item_out, child, link_abbreviations);
                }
            }
            let lines = item_lines(&item_out);
            let first_line = lines.first().copied().unwrap_or("");
            out.push_str(first_line);
            out.push('\n');
            let rest = lines.into_iter().skip(1);
            for line in rest {
                out.push_str(&format!("{}{}\n", indent_str, line));
            }
        }
    }
    out.push('\n');
}

fn item_lines(item_out: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = item_out.lines().collect();
    while lines.last().copied() == Some("") {
        lines.pop();
    }
    lines
}

fn render_inlines_org(
    children: &Option<Vec<Node>>,
    link_abbreviations: &[(String, String)],
) -> String {
    let mut out = String::new();
    if let Some(inlines) = children {
        for inline in inlines {
            match inline.r#type.as_str() {
                "text" => {
                    if let Some(val) = &inline.value {
                        out.push_str(val);
                    }
                }
                "strong" => out.push_str(&format!(
                    "*{}*",
                    render_inlines_org(&inline.children, link_abbreviations)
                )),
                "emphasis" => out.push_str(&format!(
                    "/{}/",
                    render_inlines_org(&inline.children, link_abbreviations)
                )),
                "underline" => out.push_str(&format!(
                    "_{}_",
                    render_inlines_org(&inline.children, link_abbreviations)
                )),
                "delete" => out.push_str(&format!(
                    "+{}+",
                    render_inlines_org(&inline.children, link_abbreviations)
                )),
                "inlineCode" => {
                    if let Some(val) = &inline.value {
                        out.push_str(&format!("~{}~", val));
                    }
                }
                "link" => {
                    let text = render_inlines_org(&inline.children, link_abbreviations);
                    let url = inline.get_data_str("url").unwrap_or("");
                    if let Some((abbrev, _path)) = text.split_once(':') {
                        if link_abbreviations.iter().any(|(a, _)| a == abbrev) {
                            out.push_str(&format!("[[{}]]", text));
                        } else {
                            out.push_str(&format!("[[{}][{}]]", url, text));
                        }
                    } else {
                        let mut matched = false;
                        for (abbrev, base_url) in link_abbreviations.iter() {
                            if url.starts_with(base_url) {
                                let remainder = &url[base_url.len()..];
                                out.push_str(&format!("[[{}:{}][{}]]", abbrev, remainder, text));
                                matched = true;
                                break;
                            }
                        }
                        if !matched {
                            out.push_str(&format!("[[{}][{}]]", url, text));
                        }
                    }
                }
                "image" => {
                    let alt = inline.get_data_str("alt").unwrap_or("");
                    let url = inline.get_data_str("url").unwrap_or("");
                    out.push_str(&format!("[[file:{}][{}]]", url, alt));
                }
                "break" => out.push_str("\\\\\n"),
                "html" => {
                    if let Some(val) = &inline.value {
                        out.push_str(val);
                    }
                }
                _ => {}
            }
        }
    }
    out
}
