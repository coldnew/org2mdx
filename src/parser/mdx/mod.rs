use crate::ast::Node;
use crate::error::Result;
use crate::parser::jsx;
use crate::util::iso_to_org_date;
use markdown::mdast::{AttributeContent, AttributeValue, Node as MdastNode};
use markdown::to_mdast;
use serde_json::Value;
use std::collections::HashMap;

pub fn parse_mdx(input: &str) -> Result<Node> {
    let (frontmatter, body) = extract_frontmatter(input);
    let (processed_body, exports) = extract_export_blocks(body);
    let mdast = to_mdast(&processed_body, &markdown::ParseOptions::default())
        .map_err(|e| crate::error::Error::InvalidInput(e.to_string()))?;
    let blocks = convert_node(&mdast);
    let blocks = merge_exports(blocks, &exports);
    Ok(Node::root(blocks).with_data_map(frontmatter))
}

struct ExportBlock {
    export_type: String,
    content: String,
    exports: Option<String>,
}

/// Reconstruct a JSX element string from its parsed components (name, attributes, children).
/// After the markdown parser breaks JSX into AST nodes, this rebuilds the original syntax
/// so we can store it as an "html" node for round-trip fidelity.
fn jsx_element_to_html_value(
    name: &Option<String>,
    attributes: &[AttributeContent],
    children: &[MdastNode],
) -> String {
    let name_str = name.as_deref().unwrap_or("");
    let is_fragment = name_str.is_empty();

    // Self-closing: <Name /> (no children, no attributes, not a fragment)
    if children.is_empty() && attributes.is_empty() && !is_fragment {
        return format!("<{} />", name_str);
    }

    // Build attribute string
    let mut attrs_str = String::new();
    for attr in attributes {
        match attr {
            AttributeContent::Property(prop) => {
                let val = match &prop.value {
                    Some(AttributeValue::Literal(v)) => format!("=\"{}\"", v),
                    Some(AttributeValue::Expression(expr)) => format!("={{{}}}", expr.value),
                    None => String::new(),
                };
                attrs_str.push_str(&format!(" {}{}", prop.name, val));
            }
            AttributeContent::Expression(expr) => {
                attrs_str.push_str(&format!(" {{{}}}", expr.value));
            }
        }
    }

    let open_tag = if is_fragment {
        String::from("<>")
    } else {
        format!("<{}{}>", name_str, attrs_str)
    };
    let close_tag = if is_fragment {
        String::from("</>")
    } else {
        format!("</{}>", name_str)
    };

    let children_str: String = children.iter().map(mdast_child_to_string).collect();
    format!("{}{}{}", open_tag, children_str, close_tag)
}

/// Convert a single mdast child node to string for use inside reconstructed JSX.
/// Handles text, inline formatting, nested JSX elements, and expressions.
fn mdast_child_to_string(node: &MdastNode) -> String {
    match node {
        MdastNode::Text(text) => text.value.clone(),
        MdastNode::Strong(strong) => {
            let inner: String = strong.children.iter().map(mdast_child_to_string).collect();
            format!("**{}**", inner)
        }
        MdastNode::Emphasis(em) => {
            let inner: String = em.children.iter().map(mdast_child_to_string).collect();
            format!("*{}*", inner)
        }
        MdastNode::Delete(del) => {
            let inner: String = del.children.iter().map(mdast_child_to_string).collect();
            format!("~~{}~~", inner)
        }
        MdastNode::InlineCode(code) => format!("`{}`", code.value),
        MdastNode::Link(link) => {
            let inner: String = link.children.iter().map(mdast_child_to_string).collect();
            format!("[{}]({})", inner, link.url)
        }
        MdastNode::Html(html) => html.value.clone(),
        MdastNode::MdxTextExpression(expr) => format!("{{{}}}", expr.value),
        MdastNode::MdxJsxTextElement(el) => {
            jsx_element_to_html_value(&el.name, &el.attributes, &el.children)
        }
        MdastNode::Break(_) => String::new(),
        _ => String::new(),
    }
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
        } else if jsx::is_jsx_line(trimmed) {
            // JSX block detection — find block boundaries without annotations

            // Expand backward to include preceding import/export lines
            let mut start = jsx::find_jsx_block_start(&lines, i);

            // Expand forward: include subsequent JSX lines (allow blank lines between them)
            let mut end = jsx::find_jsx_block_end(&lines, i);

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
                                        result
                                            .push(Node::new("comment").with_value(":exports none"));
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
        MdastNode::Html(html) => {
            vec![Node::new("html").with_value(&crate::parser::html::ensure_jsx(&html.value))]
        }
        MdastNode::MdxFlowExpression(expr) => {
            vec![Node::new("html").with_value(&format!("{{{}}}", expr.value))]
        }
        MdastNode::MdxJsxFlowElement(el) => {
            let value = jsx_element_to_html_value(&el.name, &el.attributes, &el.children);
            vec![Node::new("html").with_value(&value)]
        }
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
            MdastNode::Html(html) => {
                vec![Node::new("html").with_value(&crate::parser::html::ensure_jsx(&html.value))]
            }
            MdastNode::MdxTextExpression(expr) => {
                vec![Node::new("html").with_value(&format!("{{{}}}", expr.value))]
            }
            MdastNode::MdxJsxTextElement(el) => {
                let value = jsx_element_to_html_value(&el.name, &el.attributes, &el.children);
                vec![Node::new("html").with_value(&value)]
            }
            _ => vec![],
        })
        .collect()
}
