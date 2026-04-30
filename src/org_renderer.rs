use crate::ast::Node;

pub fn render_org(root: &Node) -> String {
    let mut out = String::new();
    if !root.data.is_empty() {
        let ordered_keys = [
            "title", "date", "updated", "abbrlink", "options", "tags", "language", "alias",
        ];
        let has_options = root.data.keys().any(|k| k.to_lowercase() == "options");
        for key in ordered_keys.iter() {
            if *key == "options" && !has_options {
                out.push_str("#+OPTIONS: num:nil ^:nil\n");
                continue;
            }
            if let Some(value) = root.data.get(*key) {
                render_frontmatter_org(&mut out, key, value);
            }
        }
        for key in root.data.keys() {
            let k_lower = key.to_lowercase();
            if !ordered_keys.contains(&k_lower.as_str()) {
                let value = &root.data[key];
                render_frontmatter_org(&mut out, key, value);
            }
        }
        out.push('\n');
    }
    if let Some(children) = &root.children {
        for block in children {
            render_node_org(&mut out, block);
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
            for item in list {
                if let Some(s) = item.as_str() {
                    out.push_str(&format!("#+{}: {}\n", key.to_uppercase(), s));
                }
            }
        }
        _ => {}
    }
}

fn render_node_org(out: &mut String, node: &Node) {
    match node.r#type.as_str() {
        "heading" => {
            let depth = node.get_data_num("depth").unwrap_or(1) as usize;
            let prefix = "*".repeat(depth);
            let content = render_inlines_org(&node.children);
            out.push_str(&format!("{} {}\n", prefix, content));
            let tags = node.get_data_list("tags");
            if !tags.is_empty() {
                out.push_str(&format!(" :{}:\n", tags.join(":")));
            }
        }
        "paragraph" => {
            let content = render_inlines_org(&node.children);
            if !content.is_empty() {
                if node.get_data_bool("hardLineBreak").unwrap_or(false) {
                    out.push_str(&format!("{}\\\\\n\n", content));
                } else {
                    out.push_str(&format!("{}\n\n", content));
                }
            }
        }
        "list" => render_list_org(out, node, 0),
        "code" => {
            if let Some(lang) = node.get_data_str("lang") {
                out.push_str(&format!("#+BEGIN_SRC {}\n", lang));
            } else {
                out.push_str("#+BEGIN_EXAMPLE\n");
            }
            if let Some(val) = &node.value {
                for line in val.lines() {
                    out.push_str(&format!("  {}\n", line));
                }
                if val.ends_with('\n') {
                    out.pop();
                    out.push('\n');
                }
            }
            if node.get_data_str("lang").is_some() {
                out.push_str("#+END_SRC\n\n");
            } else {
                out.push_str("#+END_EXAMPLE\n\n");
            }
        }
        "blockquote" => {
            out.push_str("#+begin_quote\n");
            if let Some(children) = &node.children {
                for child in children {
                    render_node_org(out, child);
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
        _ => {}
    }
}

fn render_list_org(out: &mut String, node: &Node, indent: usize) {
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
                    render_node_org(&mut item_out, child);
                }
            }
            let first_line = item_out.lines().next().unwrap_or("");
            out.push_str(first_line);
            out.push('\n');
            let rest: Vec<&str> = item_out.lines().skip(1).collect();
            for line in rest {
                out.push_str(&format!("{}{}\n", indent_str, line));
            }
        }
    }
    out.push('\n');
}

fn render_inlines_org(children: &Option<Vec<Node>>) -> String {
    let mut out = String::new();
    if let Some(inlines) = children {
        for inline in inlines {
            match inline.r#type.as_str() {
                "text" => {
                    if let Some(val) = &inline.value {
                        out.push_str(val);
                    }
                }
                "strong" => out.push_str(&format!("*{}*", render_inlines_org(&inline.children))),
                "emphasis" => out.push_str(&format!("/{}/", render_inlines_org(&inline.children))),
                "underline" => out.push_str(&format!("_{}_", render_inlines_org(&inline.children))),
                "delete" => out.push_str(&format!("+{}+", render_inlines_org(&inline.children))),
                "inlineCode" => {
                    if let Some(val) = &inline.value {
                        out.push_str(&format!("~{}~", val));
                    }
                }
                "link" => {
                    let text = render_inlines_org(&inline.children);
                    let url = inline.get_data_str("url").unwrap_or("");
                    out.push_str(&format!("[[{}][{}]]", url, text));
                }
                "image" => {
                    let alt = inline.get_data_str("alt").unwrap_or("");
                    let url = inline.get_data_str("url").unwrap_or("");
                    out.push_str(&format!("[[file:{}][{}]]", url, alt));
                }
                "break" => out.push_str("\\\\\n"),
                _ => {}
            }
        }
    }
    out
}
