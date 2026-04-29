use crate::ast::*;
use crate::error::Result;
use crate::util::iso_to_org_date;
use markdown::mdast::Node as MdastNode;
use markdown::to_mdast;
use std::collections::HashMap;

pub fn parse_mdx(input: &str) -> Result<Document> {
    let (frontmatter, body) = extract_frontmatter(input);
    let mdast = to_mdast(body, &markdown::ParseOptions::default())
        .map_err(|e| crate::error::Error::InvalidOrgFile(e.to_string()))?;
    let blocks = convert_node(&mdast);
    Ok(Document {
        frontmatter,
        blocks,
    })
}

fn extract_frontmatter(input: &str) -> (HashMap<String, FrontmatterValue>, &str) {
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
                                            FrontmatterValue::Str(org_date)
                                        } else {
                                            FrontmatterValue::Str(s)
                                        }
                                    } else {
                                        FrontmatterValue::Str(s)
                                    }
                                }
                                serde_yaml::Value::Sequence(seq) => {
                                    let items: Vec<String> = seq
                                        .into_iter()
                                        .filter_map(|v| {
                                            if let serde_yaml::Value::String(s) = v {
                                                Some(s)
                                            } else {
                                                let s =
                                                    serde_yaml::to_string(&v).unwrap_or_default();
                                                Some(s.trim().to_string())
                                            }
                                        })
                                        .collect();
                                    FrontmatterValue::List(items)
                                }
                                _ => {
                                    let s = serde_yaml::to_string(&v).unwrap_or_default();
                                    let s = s.trim().to_string();
                                    FrontmatterValue::Str(s)
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

fn convert_node(node: &MdastNode) -> Vec<Block> {
    match node {
        MdastNode::Root(root) => root.children.iter().flat_map(convert_node).collect(),
        MdastNode::Heading(heading) => {
            let level = heading.depth as u8;
            let content = convert_inlines(&heading.children);
            vec![Block::Heading(Heading {
                level,
                content,
                tags: vec![],
                todo_keyword: None,
                priority: None,
            })]
        }
        MdastNode::Paragraph(para) => {
            let content = convert_inlines(&para.children);
            let is_jsx = content.len() == 1
                && matches!(&content[0], Inline::Text(s) if s.trim().starts_with('{') && s.trim().ends_with('}'));
            if is_jsx {
                if let Inline::Text(s) = &content[0] {
                    vec![Block::HtmlBlock(s.trim().to_string())]
                } else {
                    unreachable!()
                }
            } else {
                vec![Block::Paragraph(Paragraph {
                    content,
                    hard_line_break: false,
                })]
            }
        }
        MdastNode::List(list) => {
            let kind = if list.ordered {
                ListKind::Ordered
            } else {
                ListKind::Unordered
            };
            let items = list
                .children
                .iter()
                .map(|item| {
                    if let MdastNode::ListItem(li) = item {
                        let content = li.children.iter().flat_map(convert_node).collect();
                        ListItem {
                            content,
                            children: vec![],
                            checkbox: None,
                        }
                    } else {
                        ListItem {
                            content: vec![],
                            children: vec![],
                            checkbox: None,
                        }
                    }
                })
                .collect();
            vec![Block::List(List { kind, items })]
        }
        MdastNode::Code(code) => {
            vec![Block::CodeBlock(CodeBlock {
                language: code.lang.clone(),
                content: code.value.clone(),
            })]
        }
        MdastNode::Blockquote(quote) => {
            let blocks = quote.children.iter().flat_map(convert_node).collect();
            vec![Block::QuoteBlock(QuoteBlock { blocks })]
        }
        MdastNode::ThematicBreak(_) => vec![Block::HorizontalRule],

        _ => vec![],
    }
}

fn convert_inlines(nodes: &[MdastNode]) -> Vec<Inline> {
    nodes
        .iter()
        .flat_map(|node| match node {
            MdastNode::Text(text) => vec![Inline::Text(text.value.clone())],
            MdastNode::Strong(strong) => vec![Inline::Bold(convert_inlines(&strong.children))],
            MdastNode::Emphasis(em) => vec![Inline::Italic(convert_inlines(&em.children))],
            MdastNode::Delete(del) => vec![Inline::StrikeThrough(convert_inlines(&del.children))],
            MdastNode::InlineCode(code) => vec![Inline::Code(code.value.clone())],
            MdastNode::Link(link) => {
                let text = convert_inlines(&link.children);
                vec![Inline::Link(Link {
                    url: link.url.clone(),
                    text,
                })]
            }
            MdastNode::Image(img) => {
                let alt_raw = img.alt.clone();
                let alt = if alt_raw.is_empty() {
                    None
                } else {
                    Some(alt_raw)
                };
                vec![Inline::Image(Image {
                    url: img.url.clone(),
                    alt_text: alt,
                })]
            }
            MdastNode::Break(_) => vec![Inline::LineBreak],
            _ => vec![],
        })
        .collect()
}
