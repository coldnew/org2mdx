use crate::ast::Node;
use crate::error::Result;
use crate::util::iso_to_org_date;
use markdown::mdast::Node as MdastNode;
use markdown::to_mdast;
use serde_json::Value;
use std::collections::HashMap;

pub fn parse_mdx(input: &str) -> Result<Node> {
    let (frontmatter, body) = extract_frontmatter(input);
    let (processed_body, exports) = extract_export_blocks(body);
    let mdast = to_mdast(&processed_body, &markdown::ParseOptions::default())
        .map_err(|e| crate::error::Error::InvalidOrgFile(e.to_string()))?;
    let blocks = convert_node(&mdast);
    let blocks = merge_exports(blocks, &exports);
    Ok(Node::root(blocks).with_data_map(frontmatter))
}

struct ExportBlock {
    export_type: String,
    content: String,
    exports: Option<String>,
}

/// Returns true if the line looks like a JSX component tag: contains `<` followed by an uppercase letter.
fn is_jsx_anchor_line(line: &str) -> bool {
    if let Some(pos) = line.find('<') {
        let after = &line[pos + 1..];
        if let Some(c) = after.chars().next() {
            return c.is_uppercase();
        }
    }
    false
}

/// Returns true if the line is a JSX anchor, or starts with `import ` / `export `.
fn is_jsx_line(line: &str) -> bool {
    let trimmed = line.trim();
    is_jsx_anchor_line(trimmed)
        || trimmed.starts_with("import ")
        || trimmed.starts_with("export ")
}

fn extract_export_blocks(body: &str) -> (String, Vec<ExportBlock>) {
    let begin_marker = "{/* #+begin_export";
    let end_marker = "{/* #+end_export";

    let lines: Vec<&str> = body.lines().collect();
    let mut result = Vec::new();
    let mut exports = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        let trimmed_lower = trimmed.to_lowercase();

        if trimmed_lower == "{/* #+begin_example */}" {
            i += 1;
            let mut content_lines = Vec::new();

            while i < lines.len() {
                if lines[i].trim().to_lowercase() == "{/* #+end_example */}" {
                    i += 1;
                    break;
                }
                content_lines.push(lines[i].to_string());
                i += 1;
            }

            let content = if content_lines.len() >= 2
                && content_lines[0].trim() == "```"
                && content_lines.last().unwrap().trim() == "```"
            {
                content_lines[1..content_lines.len() - 1].join("\n")
            } else {
                content_lines.join("\n")
            };

            let idx = exports.len();
            exports.push(ExportBlock {
                export_type: "EXAMPLE".to_string(),
                content,
                exports: None,
            });
            result.push(format!("EXPORTBLOCKPLACEHOLDER{}", idx));
        } else if trimmed.starts_with(begin_marker) && trimmed.ends_with("*/}") {
            let inner = trimmed
                .strip_prefix(begin_marker)
                .unwrap()
                .strip_suffix("*/}")
                .unwrap()
                .trim();
            let parts: Vec<&str> = inner.split_whitespace().collect();
            let export_type = parts.first().copied().unwrap_or("").to_string();
            let mut export_param = None;
            let mut j = 1;
            while j < parts.len() {
                let token = parts[j];
                if let Some(key) = token.strip_prefix(':') {
                    if key == "exports" && j + 1 < parts.len() {
                        export_param = Some(parts[j + 1].to_string());
                        j += 2;
                        continue;
                    }
                }
                j += 1;
            }

            i += 1;
            let mut content_lines = Vec::new();

            while i < lines.len() {
                let t = lines[i].trim();
                if t.starts_with(end_marker) && t.ends_with("*/}") {
                    i += 1;
                    break;
                }
                content_lines.push(lines[i].to_string());
                i += 1;
            }

            let content = content_lines.join("\n");
            let idx = exports.len();

            exports.push(ExportBlock {
                export_type,
                content,
                exports: export_param,
            });

            result.push(format!("EXPORTBLOCKPLACEHOLDER{}", idx));
        } else if is_jsx_anchor_line(trimmed) {
            // JSX block detection — find block boundaries without annotations

            // Expand backward to include preceding import/export lines
            let mut start = i;
            while start > 0 {
                let prev = lines[start - 1].trim();
                if prev.starts_with("import ") || prev.starts_with("export ") {
                    start -= 1;
                } else if prev.is_empty() && start > 1 {
                    let before_blank = lines[start - 2].trim();
                    if before_blank.starts_with("import ") || before_blank.starts_with("export ") {
                        start -= 2; // skip blank line + reach import
                        while start > 0 {
                            let more = lines[start - 1].trim();
                            if more.starts_with("import ") || more.starts_with("export ") {
                                start -= 1;
                            } else if more.is_empty() && start > 1 {
                                let bb = lines[start - 2].trim();
                                if bb.starts_with("import ") || bb.starts_with("export ") {
                                    start -= 2;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        break;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            // Expand forward: include subsequent JSX lines (allow blank lines between them)
            let mut end = i;
            while end + 1 < lines.len() {
                let next = lines[end + 1].trim();
                if is_jsx_line(next) {
                    end += 1;
                } else if next.is_empty() && end + 2 < lines.len() && is_jsx_line(lines[end + 2].trim()) {
                    end += 2; // skip blank line + reach next JSX line
                } else {
                    break;
                }
            }

            // Trim leading/trailing blank lines from the block
            while start < end && lines[start].trim().is_empty() {
                start += 1;
            }
            while end > start && lines[end].trim().is_empty() {
                end -= 1;
            }

            let block_lines: Vec<&str> = lines[start..=end].to_vec();
            let content = block_lines.join("\n");

            if !content.trim().is_empty() {
                let idx = exports.len();
                exports.push(ExportBlock {
                    export_type: "jsx".to_string(),
                    content,
                    exports: None,
                });
                result.push(format!("EXPORTBLOCKPLACEHOLDER{}", idx));
            } else {
                // Fallback: emit original lines unchanged
                for j in i..=end {
                    result.push(lines[j].to_string());
                }
            }
            i = end + 1;
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }

    (result.join("\n"), exports)
}

fn merge_exports(blocks: Vec<Node>, exports: &[ExportBlock]) -> Vec<Node> {
    let mut result = Vec::new();
    for block in blocks {
        let mut handled = false;
        if block.r#type == "paragraph" {
            if let Some(children) = &block.children {
                for child in children {
                    if child.r#type == "text" {
                        if let Some(val) = &child.value {
                            let trimmed = val.trim();
                            for (idx, exp) in exports.iter().enumerate() {
                                let placeholder = format!("EXPORTBLOCKPLACEHOLDER{}", idx);
                                if trimmed == placeholder {
                                    if exp.export_type == "EXAMPLE" {
                                        result.push(
                                            Node::new("code")
                                                .with_value(&exp.content)
                                                .data_str("block_type", "example"),
                                        );
                                    } else if exp.exports.as_deref() == Some("none") {
                                        result.push(Node::new("comment").with_value(":exports none"));
                                    } else {
                                        result.push(
                                            Node::new("export")
                                                .with_value(&exp.content)
                                                .data_str("lang", &exp.export_type),
                                        );
                                    }
                                    handled = true;
                                    break;
                                }
                            }
                        }
                    }
                    if handled {
                        break;
                    }
                }
            }
        }
        if !handled {
            result.push(block);
        }
    }
    result
}

fn extract_frontmatter(input: &str) -> (HashMap<String, Value>, &str) {
    let input = input.trim_start();
    if !input.starts_with("---\n") {
        return (HashMap::new(), input);
    }
    let after_start = &input[4..];
    if let Some(end) = after_start.find("\n---") {
        let yaml_str = &after_start[..end];
        let rest = &after_start[end + 4..];
        match serde_yaml::from_str::<serde_yaml::Value>(yaml_str) {
            Ok(value) => {
                let mut map = HashMap::new();
                if let serde_yaml::Value::Mapping(mapping) = value {
                    for (k, v) in mapping {
                        if let serde_yaml::Value::String(key) = k {
                            let val = match v {
                                serde_yaml::Value::String(s) => {
                                    let key_lower = key.to_lowercase();
                                    if key_lower == "date" || key_lower == "updated" {
                                        if let Some(org_date) = iso_to_org_date(&s) {
                                            Value::String(org_date)
                                        } else {
                                            Value::String(s)
                                        }
                                    } else {
                                        Value::String(s)
                                    }
                                }
                                serde_yaml::Value::Sequence(seq) => {
                                    let items: Vec<Value> = seq
                                        .into_iter()
                                        .map(|v| {
                                            if let serde_yaml::Value::String(s) = v {
                                                Value::String(s)
                                            } else {
                                                let s =
                                                    serde_yaml::to_string(&v).unwrap_or_default();
                                                Value::String(s.trim().to_string())
                                            }
                                        })
                                        .collect();
                                    Value::Array(items)
                                }
                                serde_yaml::Value::Mapping(mapping) => {
                                    convert_yaml_mapping_to_json(mapping)
                                }
                                _ => {
                                    let s = serde_yaml::to_string(&v).unwrap_or_default();
                                    Value::String(s.trim().to_string())
                                }
                            };
                            map.insert(key, val);
                        }
                    }
                }
                (map, rest.trim_start())
            }
            Err(_) => (HashMap::new(), input),
        }
    } else {
        (HashMap::new(), input)
    }
}

fn convert_yaml_mapping_to_json(mapping: serde_yaml::Mapping) -> Value {
    let mut json_map = serde_json::Map::new();
    for (k, v) in mapping {
        if let serde_yaml::Value::String(key_str) = k {
            let val = match v {
                serde_yaml::Value::String(s) => Value::String(s),
                serde_yaml::Value::Mapping(inner) => convert_yaml_mapping_to_json(inner),
                serde_yaml::Value::Sequence(seq) => {
                    let items: Vec<Value> = seq
                        .into_iter()
                        .map(|item| match item {
                            serde_yaml::Value::String(s) => Value::String(s),
                            serde_yaml::Value::Mapping(m) => convert_yaml_mapping_to_json(m),
                            other => {
                                let s = serde_yaml::to_string(&other).unwrap_or_default();
                                Value::String(s.trim().to_string())
                            }
                        })
                        .collect();
                    Value::Array(items)
                }
                serde_yaml::Value::Bool(b) => Value::Bool(b),
                serde_yaml::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Value::Number(serde_json::Number::from(i))
                    } else if let Some(f) = n.as_f64() {
                        serde_json::Number::from_f64(f)
                            .map(Value::Number)
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    }
                }
                serde_yaml::Value::Null | serde_yaml::Value::Tagged(_) => Value::Null,
            };
            json_map.insert(key_str, val);
        }
    }
    Value::Object(json_map)
}

fn convert_node(node: &MdastNode) -> Vec<Node> {
    match node {
        MdastNode::Root(root) => root.children.iter().flat_map(convert_node).collect(),
        MdastNode::Heading(heading) => {
            let content = convert_inlines(&heading.children);
            vec![Node::new("heading")
                .with_children(content)
                .data_num("depth", heading.depth as u64)]
        }
        MdastNode::Paragraph(para) => {
            let content = convert_inlines(&para.children);
            let is_jsx = content.len() == 1
                && content[0].r#type == "text"
                && content[0]
                    .value
                    .as_deref()
                    .map(|s| s.trim().starts_with('{') && s.trim().ends_with('}'))
                    .unwrap_or(false);
            if is_jsx {
                if let Some(val) = &content[0].value {
                    vec![Node::new("html").with_value(val.trim())]
                } else {
                    vec![]
                }
            } else {
                vec![Node::new("paragraph").with_children(content)]
            }
        }
        MdastNode::List(list) => {
            let ordered = list.ordered;
            let items: Vec<Node> = list
                .children
                .iter()
                .map(|item| {
                    if let MdastNode::ListItem(li) = item {
                        let content: Vec<Node> =
                            li.children.iter().flat_map(convert_node).collect();
                        Node::new("listItem").with_children(content)
                    } else {
                        Node::new("listItem")
                    }
                })
                .collect();
            vec![Node::new("list")
                .with_children(items)
                .data_bool("ordered", ordered)]
        }
        MdastNode::Code(code) => {
            let mut node = Node::new("code").with_value(&code.value);
            if let Some(ref lang) = code.lang {
                node.data
                    .insert("lang".to_string(), Value::String(lang.clone()));
            }
            vec![node]
        }
        MdastNode::Blockquote(quote) => {
            let children: Vec<Node> = quote.children.iter().flat_map(convert_node).collect();
            vec![Node::new("blockquote").with_children(children)]
        }
        MdastNode::ThematicBreak(_) => vec![Node::new("thematicBreak")],
        _ => vec![],
    }
}

fn convert_inlines(nodes: &[MdastNode]) -> Vec<Node> {
    nodes
        .iter()
        .flat_map(|node| match node {
            MdastNode::Text(text) => vec![Node::text(&text.value)],
            MdastNode::Strong(strong) => {
                vec![Node::new("strong").with_children(convert_inlines(&strong.children))]
            }
            MdastNode::Emphasis(em) => {
                vec![Node::new("emphasis").with_children(convert_inlines(&em.children))]
            }
            MdastNode::Delete(del) => {
                vec![Node::new("delete").with_children(convert_inlines(&del.children))]
            }
            MdastNode::InlineCode(code) => {
                vec![Node::new("inlineCode").with_value(&code.value)]
            }
            MdastNode::Link(link) => {
                let text = convert_inlines(&link.children);
                vec![Node::new("link")
                    .with_children(text)
                    .data_str("url", &link.url)]
            }
            MdastNode::Image(img) => {
                let alt = if img.alt.is_empty() {
                    String::new()
                } else {
                    img.alt.clone()
                };
                vec![Node::new("image")
                    .data_str("url", &img.url)
                    .data_str("alt", &alt)]
            }
            MdastNode::Break(_) => vec![Node::new("break")],
            _ => vec![],
        })
        .collect()
}
