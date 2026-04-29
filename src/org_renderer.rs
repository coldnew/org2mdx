use crate::ast::*;

pub fn render_org(doc: &Document) -> String {
    let mut out = String::new();
    if !doc.frontmatter.is_empty() {
        out.push_str("---\n");
        for (key, value) in &doc.frontmatter {
            match value {
                FrontmatterValue::Str(s) => {
                    out.push_str(&format!("#+{}: {}\n", key.to_uppercase(), s))
                }
                FrontmatterValue::List(list) => {
                    out.push_str(&format!("#+{}: {}\n", key.to_uppercase(), list.join(", ")));
                }
            }
        }
        out.push_str("---\n\n");
    }
    for block in &doc.blocks {
        render_block_org(&mut out, block);
    }
    out
}

fn render_block_org(out: &mut String, block: &Block) {
    match block {
        Block::Heading(h) => {
            let prefix = "*".repeat(h.level as usize);
            let content = render_inline_org(&h.content);
            out.push_str(&format!("{} {}\n", prefix, content));
            if !h.tags.is_empty() {
                out.push_str(&format!(" :{}:\n", h.tags.join(":")));
            }
        }
        Block::Paragraph(p) => {
            let content = render_inline_org(&p.content);
            if !content.is_empty() {
                if p.hard_line_break {
                    out.push_str(&format!("{}\\\\\n\n", content));
                } else {
                    out.push_str(&format!("{}\n\n", content));
                }
            }
        }
        Block::List(list) => render_list_org(out, list, 0),
        Block::CodeBlock(cb) => {
            if let Some(_lang) = &cb.language {
                out.push_str(&format!("#+begin_src {}\n", _lang));
            } else {
                out.push_str("#+begin_example\n");
            }
            out.push_str(&cb.content);
            if !cb.content.ends_with('\n') {
                out.push('\n');
            }
            if let Some(_lang) = &cb.language {
                out.push_str(&format!("#+begin_src {}\n", _lang));
            } else {
                out.push_str("#+begin_example\n");
            }
        }
        Block::QuoteBlock(qb) => {
            out.push_str("#+begin_quote\n");
            for block in &qb.blocks {
                render_block_org(out, block);
            }
            out.push_str("#+end_quote\n\n");
        }
        Block::HorizontalRule => out.push_str("----\n\n"),
        Block::BlankLine => out.push('\n'),
        Block::HtmlBlock(html) => {
            out.push_str(&format!("#+HTML: {}\n", html));
        }
    }
}

fn render_list_org(out: &mut String, list: &List, indent: usize) {
    let indent_str = "  ".repeat(indent);
    for (i, item) in list.items.iter().enumerate() {
        match list.kind {
            ListKind::Unordered => out.push_str(&format!("{}- ", indent_str)),
            ListKind::Ordered => out.push_str(&format!("{}{}. ", indent_str, i + 1)),
            ListKind::Description => out.push_str(&format!("{}- ", indent_str)),
        }
        let mut item_out = String::new();
        for block in &item.content {
            render_block_org(&mut item_out, block);
        }
        let first_line = item_out.lines().next().unwrap_or("");
        out.push_str(first_line);
        out.push('\n');
        for child in &item.children {
            render_list_org(
                out,
                &List {
                    kind: list.kind.clone(),
                    items: vec![child.clone()],
                },
                indent + 1,
            );
        }
        let rest: Vec<&str> = item_out.lines().skip(1).collect();
        for line in rest {
            out.push_str(&format!("{}{}\n", indent_str, line));
        }
    }
    out.push('\n');
}

fn render_inline_org(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(s) => out.push_str(s),
            Inline::Bold(children) => out.push_str(&format!("*{}*", render_inline_org(children))),
            Inline::Italic(children) => out.push_str(&format!("/{}/", render_inline_org(children))),
            Inline::Underline(children) => {
                out.push_str(&format!("_{}_", render_inline_org(children)))
            }
            Inline::StrikeThrough(children) => {
                out.push_str(&format!("+{}+", render_inline_org(children)))
            }
            Inline::Code(s) => out.push_str(&format!("~{}~", s)),
            Inline::Verbatim(s) => out.push_str(&format!("={}=", s)),
            Inline::Link(link) => {
                let text = render_inline_org(&link.text);
                out.push_str(&format!("[[{}][{}]]", link.url, text));
            }
            Inline::Image(img) => {
                let alt = img.alt_text.as_deref().unwrap_or("");
                out.push_str(&format!("[[file:{}][{}]]", img.url, alt));
            }
            Inline::LineBreak => out.push_str("\\\\\n"),
        }
    }
    out
}
