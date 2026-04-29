use crate::ast::*;
use crate::error::Result;
use markdown::mdast::Node as MdastNode;
use markdown::to_mdast;

pub fn parse_mdx(input: &str) -> Result<Document> {
    let mdast = to_mdast(input, &markdown::ParseOptions::default())
        .map_err(|e| crate::error::Error::InvalidOrgFile(e.to_string()))?;
    let blocks = convert_node(&mdast);
    Ok(Document {
        frontmatter: Default::default(), // TODO: extract frontmatter from MDX
        blocks,
    })
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
            vec![Block::Paragraph(Paragraph {
                content,
                hard_line_break: false,
            })]
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
