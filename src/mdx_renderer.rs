use crate::ast::Node;

pub fn render_mdx(root: &Node) -> String {
    let mut out = String::new();
    if !root.data.is_empty() {
        out.push_str("---\n");
        let ordered_keys = [
            "title", "date", "updated", "abbrlink", "tags", "language", "alias",
        ];
        for key in &ordered_keys {
            if let Some(value) = root.data.get(*key) {
                render_frontmatter_value(&mut out, key, value);
            }
        }
        for (key, value) in &root.data {
            if !ordered_keys.contains(&key.as_str()) {
                render_frontmatter_value(&mut out, key, value);
            }
        }
        out.push_str("---\n");
    }
    if let Some(children) = &root.children {
        for child in children {
            render_node(&mut out, child);
        }
    }
    out
}

fn render_frontmatter_value(out: &mut String, key: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            if key == "abbrlink" {
                out.push_str(&format!("{}: {}\n", key, s));
            } else {
                out.push_str(&format!("{}: {}\n", key, crate::util::yaml_str(s)));
            }
        }
        serde_json::Value::Array(list) => {
            out.push_str(&format!("{}:\n", key));
            for item in list {
                if let Some(s) = item.as_str() {
                    out.push_str(&format!("  - {}\n", s));
                }
            }
        }
        _ => {}
    }
}

fn render_node(out: &mut String, node: &Node) {
    match node.r#type.as_str() {
        "heading" => {
            let depth = node.get_data_num("depth").unwrap_or(1).min(6) as usize;
            let prefix = "#".repeat(depth);
            let content = render_inlines(&node.children);
            out.push_str(&format!("{} {}\n", prefix, content));
        }
        "paragraph" => {
            let content = render_inlines(&node.children);
            if !content.is_empty() {
                if node.get_data_bool("hardLineBreak").unwrap_or(false) {
                    out.push_str(&format!("{}\\\\\n", content));
                } else {
                    out.push_str(&format!("{}\n", content));
                }
            }
        }
        "list" => render_list(out, node, 0),
        "code" => {
            if let Some(lang) = node.get_data_str("lang") {
                out.push_str(&format!("```{}\n", lang));
            } else {
                out.push_str("```\n");
            }
            if let Some(val) = &node.value {
                out.push_str(val);
                if !val.ends_with('\n') {
                    out.push('\n');
                }
            }
            out.push_str("```\n");
        }
        "blockquote" => {
            if let Some(children) = &node.children {
                for child in children {
                    let mut line = String::new();
                    render_node(&mut line, child);
                    for l in line.lines() {
                        out.push_str(&format!("> {}\n", l));
                    }
                    out.push('\n');
                }
            }
        }
        "thematicBreak" => out.push_str("---\n\n"),
        "blankLine" => out.push('\n'),
        "html" => {
            if let Some(val) = &node.value {
                out.push_str(val);
                out.push('\n');
            }
        }
        _ => {}
    }
}

fn render_list(out: &mut String, node: &Node, indent: usize) {
    let indent_str = "  ".repeat(indent);
    let ordered = node.get_data_bool("ordered").unwrap_or(false);
    if let Some(items) = &node.children {
        for (i, item) in items.iter().enumerate() {
            if ordered {
                out.push_str(&format!("{}{}. ", indent_str, i + 1));
            } else {
                out.push_str(&format!("{}* ", indent_str));
            }
            let mut item_out = String::new();
            if let Some(children) = &item.children {
                for child in children {
                    render_node(&mut item_out, child);
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

fn render_inlines(children: &Option<Vec<Node>>) -> String {
    let mut out = String::new();
    if let Some(inlines) = children {
        for inline in inlines {
            match inline.r#type.as_str() {
                "text" => {
                    if let Some(val) = &inline.value {
                        out.push_str(val);
                    }
                }
                "strong" => out.push_str(&format!("**{}**", render_inlines(&inline.children))),
                "emphasis" => out.push_str(&format!("*{}*", render_inlines(&inline.children))),
                "underline" => {
                    out.push_str(&format!("<u>{}</u>", render_inlines(&inline.children)))
                }
                "delete" => out.push_str(&format!("~~{}~~", render_inlines(&inline.children))),
                "inlineCode" => {
                    if let Some(val) = &inline.value {
                        out.push_str(&format!("`{}`", val));
                    }
                }
                "link" => {
                    let text = render_inlines(&inline.children);
                    let url = inline.get_data_str("url").unwrap_or("");
                    out.push_str(&format!("[{}]({})", text, url));
                }
                "image" => {
                    let alt = inline.get_data_str("alt").unwrap_or("");
                    let url = inline.get_data_str("url").unwrap_or("");
                    out.push_str(&format!("[{}]({})", alt, url));
                }
                "break" => out.push_str("  \n"),
                _ => {}
            }
        }
    }
    out
}
