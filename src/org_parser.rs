use crate::ast::*;
use crate::error::Result;
// // // // use crate::heading::split_tags;
use crate::inline_parser::parse_inline;
use crate::util::kw;
use std::collections::HashMap;

pub struct OrgParser {
    lines: Vec<String>,
    pos: usize,
    link_aliases: HashMap<String, String>,
    frontmatter: HashMap<String, FrontmatterValue>,
}

impl OrgParser {
    pub fn new(input: &str) -> Self {
        Self {
            lines: input.lines().map(|l| l.to_string()).collect(),
            pos: 0,
            link_aliases: HashMap::new(),
            frontmatter: HashMap::new(),
        }
    }

    fn peek(&self) -> Option<&str> {
        self.lines.get(self.pos).map(|s| s.as_str())
    }

    fn advance(&mut self) -> Option<&str> {
        let l = self.lines.get(self.pos).map(|s| s.as_str());
        if l.is_some() {
            self.pos += 1;
        }
        l
    }

    pub fn parse(mut self) -> Result<Document> {
        // First pass: collect directives
        for line in &self.lines.clone() {
            self.process_directive(line);
        }
        self.pos = 0;
        let blocks = self.parse_blocks()?;
        Ok(Document {
            frontmatter: self.frontmatter,
            blocks,
        })
    }

    fn process_directive(&mut self, line: &str) {
        let trimmed = line.trim();
        if let Some(v) = kw(trimmed, "TITLE") {
            self.frontmatter
                .insert("title".to_string(), FrontmatterValue::Str(v.to_string()));
        } else if let Some(v) = kw(trimmed, "DATE") {
            if let Some(iso) = crate::util::org_date_to_iso(v) {
                self.frontmatter
                    .insert("date".to_string(), FrontmatterValue::Str(iso));
            }
        } else if let Some(v) = kw(trimmed, "UPDATED") {
            if let Some(iso) = crate::util::org_date_to_iso(v) {
                self.frontmatter
                    .insert("updated".to_string(), FrontmatterValue::Str(iso));
            }
        } else if let Some(v) = kw(trimmed, "ABBRLINK") {
            self.frontmatter
                .insert("abbrlink".to_string(), FrontmatterValue::Str(v.to_string()));
        } else if let Some(v) = kw(trimmed, "TAGS") {
            let items: Vec<String> = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            self.frontmatter
                .insert("tags".to_string(), FrontmatterValue::List(items));
        } else if let Some(v) = kw(trimmed, "CATEGORY") {
            self.frontmatter.insert(
                "category".to_string(),
                FrontmatterValue::List(vec![v.to_string()]),
            );
        } else if let Some(v) = kw(trimmed, "AUTHOR") {
            self.frontmatter
                .insert("author".to_string(), FrontmatterValue::Str(v.to_string()));
        } else if let Some(v) = kw(trimmed, "EMAIL") {
            self.frontmatter.insert(
                "author_email".to_string(),
                FrontmatterValue::Str(v.to_string()),
            );
        } else if let Some(v) = kw(trimmed, "ALIAS") {
            let list = self
                .frontmatter
                .entry("alias".to_string())
                .or_insert(FrontmatterValue::List(vec![]));
            if let FrontmatterValue::List(ref mut l) = list {
                l.push(v.to_string());
            }
        } else if let Some(v) = kw(trimmed, "LANGUAGE") {
            self.frontmatter
                .insert("language".to_string(), FrontmatterValue::Str(v.to_string()));
        } else if let Some(v) = kw(trimmed, "ATTR_HTML") {
            self.frontmatter.insert(
                "attr_html".to_string(),
                FrontmatterValue::Str(v.to_string()),
            );
        } else if let Some(v) = kw(trimmed, "LINK") {
            if let Some(sp) = v.find(char::is_whitespace) {
                let name = v[..sp].trim().to_string();
                let url = v[sp..].trim().to_string();
                self.link_aliases.insert(name, url);
            }
        }
    }

    fn parse_blocks(&mut self) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos].clone();
            let trimmed = line.trim();
            if trimmed.is_empty() {
                blocks.push(Block::BlankLine);
                self.advance();
                continue;
            }
            // Handle headings
            if let Some((depth, title, tags)) = crate::heading::parse_heading(&line) {
                if crate::heading::should_skip_section(&tags) {
                    self.skip_section(depth);
                    continue;
                }
                let content = parse_inline(title, &self.link_aliases);
                let heading = Heading {
                    level: depth as u8,
                    content,
                    tags: tags.into_iter().map(|s| s.to_string()).collect(),
                    todo_keyword: None,
                    priority: None,
                };
                blocks.push(Block::Heading(heading));
                self.advance();
                continue;
            }
            // Handle code blocks (#+begin_src etc)
            let tl_lower = trimmed.to_lowercase();
            if tl_lower.starts_with("#+begin_src")
                || tl_lower.starts_with("#+begin_example")
                || tl_lower.starts_with("#+begin_quote")
                || tl_lower.starts_with("#+begin_center")
                || tl_lower.starts_with("#+begin_export")
                || tl_lower.starts_with("#+begin_")
            {
                let block = self.parse_block(&line)?;
                blocks.push(block);
                continue;
            }
            // Handle : prefix example blocks
            if line.starts_with(": ") || line == ":" {
                let code = self.parse_colon_block();
                blocks.push(Block::CodeBlock(CodeBlock {
                    language: None,
                    content: code,
                }));
                continue;
            }
            // Handle unordered list
            if crate::list::is_unordered_item(&line) {
                let list = self.parse_unordered_list()?;
                blocks.push(Block::List(list));
                continue;
            }
            // Handle ordered list
            if crate::list::is_ordered_item(&line) {
                let list = self.parse_ordered_list()?;
                blocks.push(Block::List(list));
                continue;
            }
            // Handle paragraphs
            let (para, _had_lb) = self.parse_paragraph();
            if !para.content.is_empty() {
                blocks.push(Block::Paragraph(para));
            }
        }
        Ok(blocks)
    }

    fn skip_section(&mut self, heading_level: u32) {
        self.advance();
        while let Some(line) = self.peek() {
            if let Some((lvl, _, _)) = crate::heading::parse_heading(line) {
                if lvl <= heading_level {
                    break;
                }
            }
            self.advance();
        }
    }

    fn parse_block(&mut self, start_line: &str) -> Result<Block> {
        let block_type = crate::block::parse_block_begin(start_line);
        self.advance();
        let end_kw = block_type.end_keyword();
        let mut lines = Vec::new();
        while let Some(line) = self.peek() {
            if line
                .trim()
                .to_lowercase()
                .starts_with(&format!("#+end_{}", end_kw))
            {
                self.advance();
                break;
            }
            lines.push(line.to_string());
            self.advance();
        }
        match block_type {
            crate::block::BlockType::Src(lang) => {
                let content = lines.join("\n");
                Ok(Block::CodeBlock(CodeBlock {
                    language: Some(lang),
                    content,
                }))
            }
            crate::block::BlockType::Example => {
                let content = lines.join("\n");
                Ok(Block::CodeBlock(CodeBlock {
                    language: None,
                    content,
                }))
            }
            crate::block::BlockType::Quote => {
                let mut quote_blocks = Vec::new();
                for line in lines {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        quote_blocks.push(Block::BlankLine);
                    } else {
                        let para = Paragraph {
                            content: parse_inline(trimmed, &self.link_aliases),
                            hard_line_break: false,
                        };
                        quote_blocks.push(Block::Paragraph(para));
                    }
                }
                Ok(Block::QuoteBlock(QuoteBlock {
                    blocks: quote_blocks,
                }))
            }
            crate::block::BlockType::Center => {
                let content = lines.join(" ");
                let para = Paragraph {
                    content: parse_inline(&content, &self.link_aliases),
                    hard_line_break: false,
                };
                Ok(Block::Paragraph(para))
            }
            crate::block::BlockType::Export => Ok(Block::HtmlBlock(String::new())),
            crate::block::BlockType::Unknown(_) => {
                let content = lines.join(" ");
                let para = Paragraph {
                    content: parse_inline(&content, &self.link_aliases),
                    hard_line_break: false,
                };
                Ok(Block::Paragraph(para))
            }
        }
    }

    fn parse_colon_block(&mut self) -> String {
        let mut lines = Vec::new();
        while let Some(line) = self.peek() {
            if line.starts_with(": ") {
                lines.push(line[2..].to_string());
                self.advance();
            } else if line == ":" {
                lines.push(String::new());
                self.advance();
            } else {
                break;
            }
        }
        lines.join("\n")
    }

    fn parse_paragraph(&mut self) -> (Paragraph, bool) {
        let mut parts = Vec::new();
        let mut had_line_break = false;
        while let Some(line) = self.peek() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if trimmed.starts_with("#+") || trimmed.starts_with("#-") {
                break;
            }
            if crate::heading::parse_heading(line).is_some() {
                break;
            }
            if crate::list::is_unordered_item(line) || crate::list::is_ordered_item(line) {
                break;
            }
            let (text, is_lb) = if trimmed.ends_with("\\\\") {
                (trimmed[..trimmed.len() - 2].trim_end().to_string(), true)
            } else {
                (trimmed.to_string(), false)
            };
            if is_lb {
                had_line_break = true;
            }
            parts.push(text);
            self.advance();
        }
        let joined = parts.join("\n");
        let normalized = crate::util::collapse_spaces(&joined);
        let content = parse_inline(&normalized, &self.link_aliases);
        (
            Paragraph {
                content,
                hard_line_break: had_line_break,
            },
            had_line_break,
        )
    }

    fn parse_unordered_list(&mut self) -> Result<List> {
        let mut items = Vec::new();
        while let Some(line) = self.peek() {
            if !crate::list::is_unordered_item(line) {
                break;
            }
            let content_raw = crate::list::unordered_content(line);
            let content_inline = parse_inline(content_raw.trim(), &self.link_aliases);
            let mut item_blocks = vec![Block::Paragraph(Paragraph {
                content: content_inline,
                hard_line_break: false,
            })];
            self.advance();
            // Check for nested content (indented lines)
            let nested = self.parse_list_nested()?;
            if !nested.items.is_empty() {
                item_blocks.push(Block::List(nested));
            }
            items.push(ListItem {
                content: item_blocks,
                children: vec![],
                checkbox: None,
            });
        }
        Ok(List {
            kind: ListKind::Unordered,
            items,
        })
    }

    fn parse_ordered_list(&mut self) -> Result<List> {
        let mut items = Vec::new();
        while let Some(line) = self.peek() {
            if !crate::list::is_ordered_item(line) {
                break;
            }
            let (_, content_raw) = crate::list::ordered_parts(line);
            let content_inline = parse_inline(content_raw.trim(), &self.link_aliases);
            let mut item_blocks = vec![Block::Paragraph(Paragraph {
                content: content_inline,
                hard_line_break: false,
            })];
            self.advance();
            let nested = self.parse_list_nested()?;
            if !nested.items.is_empty() {
                item_blocks.push(Block::List(nested));
            }
            items.push(ListItem {
                content: item_blocks,
                children: vec![],
                checkbox: None,
            });
        }
        Ok(List {
            kind: ListKind::Ordered,
            items,
        })
    }

    fn parse_list_nested(&mut self) -> Result<List> {
        let mut items = Vec::new();
        while let Some(line) = self.peek() {
            if !line.starts_with("  ") {
                break;
            }
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                self.advance();
                continue;
            }
            if crate::list::is_unordered_item(trimmed) {
                let content_raw = crate::list::unordered_content(trimmed);
                let content_inline = parse_inline(content_raw.trim(), &self.link_aliases);
                items.push(ListItem {
                    content: vec![Block::Paragraph(Paragraph {
                        content: content_inline,
                        hard_line_break: false,
                    })],
                    children: vec![],
                    checkbox: None,
                });
                self.advance();
            } else if crate::list::is_ordered_item(trimmed) {
                let (_, content_raw) = crate::list::ordered_parts(trimmed);
                let content_inline = parse_inline(content_raw.trim(), &self.link_aliases);
                items.push(ListItem {
                    content: vec![Block::Paragraph(Paragraph {
                        content: content_inline,
                        hard_line_break: false,
                    })],
                    children: vec![],
                    checkbox: None,
                });
                self.advance();
            } else {
                break;
            }
        }
        Ok(List {
            kind: ListKind::Unordered,
            items,
        })
    }
}

pub fn parse_org(input: &str) -> Result<Document> {
    OrgParser::new(input).parse()
}
