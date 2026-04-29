use crate::ast::*;

pub fn render_mdx(doc: &Document) -> String {
    let mut out = String::new();
    // Frontmatter
    if !doc.frontmatter.is_empty() {
        out.push_str("---\n");
        for (key, value) in &doc.frontmatter {
            match value {
                FrontmatterValue::Str(s) => {
                    out.push_str(&format!("{}: {}\n", key, s));
                }
                FrontmatterValue::List(list) => {
                    out.push_str(&format!("{}:\n", key));
                    for item in list {
                        out.push_str(&format!("  - {}\n", item));
                    }
                }
            }
        }
        out.push_str("---\n\n");
    }
    for block in &doc.blocks {
        render_block(&mut out, block);
    }
    out
}

fn render_block(out: &mut String, block: &Block) {
    match block {
        Block::Heading(h) => {
            let level = h.level.min(6);
            let prefix = "#".repeat(level as usize);
            let content = render_inline_vec(&h.content);
            out.push_str(&format!("{} {}\n", prefix, content));
            if !h.tags.is_empty() {
                // Tags are not typically rendered in MDX, but we can add as comment
                // out.push_str(&format!("<!-- tags: {} -->\n", h.tags.join(", ")));
            }
        }
        Block::Paragraph(p) => {
            let content = render_inline_vec(&p.content);
            if !content.is_empty() {
                if p.hard_line_break {
                    out.push_str(&format!("{}\\\\ \n", content));
                } else {
                    out.push_str(&format!("{}\n\n", content));
                }
            }
        }
        Block::List(list) => render_list(out, list, 0),
        Block::CodeBlock(cb) => {
            if let Some(lang) = &cb.language {
                out.push_str(&format!("```{}\n", lang));
            } else {
                out.push_str("```\n");
            }
            out.push_str(&cb.content);
            if !cb.content.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n\n");
        }
        Block::QuoteBlock(qb) => {
            for block in &qb.blocks {
                let mut line = String::new();
                render_block(&mut line, block);
                for l in line.lines() {
                    out.push_str(&format!("> {}\n", l));
                }
                out.push('\n');
            }
        }
        Block::HorizontalRule => out.push_str("---\n\n"),
        Block::BlankLine => out.push('\n'),
        Block::HtmlBlock(html) => {
            out.push_str(html);
            out.push('\n');
        }
    }
}

fn render_list(out: &mut String, list: &List, indent: usize) {
    let indent_str = "  ".repeat(indent);
    for (i, item) in list.items.iter().enumerate() {
        match list.kind {
            ListKind::Unordered => {
                out.push_str(&format!("{}* ", indent_str));
            }
            ListKind::Ordered => {
                out.push_str(&format!("{}{}. ", indent_str, i + 1));
            }
            ListKind::Description => {
                // Not implemented yet
                out.push_str(&format!("{}- ", indent_str));
            }
        }
        // Render item content
        let mut item_out = String::new();
        for block in &item.content {
            render_block(&mut item_out, block);
        }
        let first_line = item_out.lines().next().unwrap_or("");
        out.push_str(first_line);
        out.push('\n');
        // Handle children (nested lists)
        for child in &item.children {
            render_list(
                out,
                &List {
                    kind: list.kind.clone(),
                    items: vec![child.clone()],
                },
                indent + 1,
            );
        }
        // Remaining lines of content
        let rest: Vec<&str> = item_out.lines().skip(1).collect();
        for line in rest {
            out.push_str(&format!("{}{}\n", indent_str, line));
        }
    }
    out.push('\n');
}

fn render_inline_vec(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(s) => out.push_str(s),
            Inline::Bold(children) => out.push_str(&format!("**{}**", render_inline_vec(children))),
            Inline::Italic(children) => out.push_str(&format!("*{}*", render_inline_vec(children))),
            Inline::Underline(children) => {
                out.push_str(&format!("<u>{}</u>", render_inline_vec(children)))
            }
            Inline::StrikeThrough(children) => {
                out.push_str(&format!("~~{}~~", render_inline_vec(children)))
            }
            Inline::Code(s) => out.push_str(&format!("`{}`", s)),
            Inline::Verbatim(s) => out.push_str(&format!("`{}`", s)),
            Inline::Link(link) => {
                let text = render_inline_vec(&link.text);
                out.push_str(&format!("[{}]({})", text, link.url));
            }
            Inline::Image(img) => {
                let alt = img.alt_text.as_deref().unwrap_or("");
                out.push_str(&format!("![{}]({})", alt, img.url));
            }
            Inline::LineBreak => out.push_str("  \n"),
        }
    }
    out
}
