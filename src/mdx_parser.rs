use crate::ast::Node;
use crate::error::Result;
use crate::util::iso_to_org_date;
use markdown::mdast::Node as MdastNode;
use markdown::to_mdast;
use serde_json::Value;
use std::collections::HashMap;

pub fn parse_mdx(input: &str) -> Result<Node> {
    let (frontmatter, body) = extract_frontmatter(input);
    let mdast = to_mdast(body, &markdown::ParseOptions::default())
        .map_err(|e| crate::error::Error::InvalidOrgFile(e.to_string()))?;
    let blocks = convert_node(&mdast);
    Ok(Node::root(blocks).with_data_map(frontmatter))
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
                .data_num("depth", heading.depth as u8)]
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
