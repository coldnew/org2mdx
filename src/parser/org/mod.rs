pub mod block;
pub mod heading;
pub mod list;

use crate::ast::Node;
use crate::error::Result;
use crate::parser::inline::parse_inline;
use crate::util::kw;
use serde_json::Value;
use std::collections::HashMap;

pub struct OrgParser {
    lines: Vec<String>,
    pos: usize,
    link_aliases: HashMap<String, String>,
    frontmatter: HashMap<String, Value>,
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

    pub fn parse(mut self) -> Result<Node> {
        for line in &self.lines.clone() {
            self.process_directive(line);
        }
        self.pos = 0;
        let blocks = self.parse_blocks()?;
        Ok(Node::root(blocks).with_data_map(self.frontmatter))
    }

    fn process_directive(&mut self, line: &str) {
        let trimmed = line.trim();
        if let Some(v) = kw(trimmed, "TITLE") {
            self.frontmatter
                .insert("title".to_string(), Value::String(v.to_string()));
        } else if let Some(v) = kw(trimmed, "DATE") {
            if let Some(iso) = crate::util::org_date_to_iso(v) {
                self.frontmatter
                    .insert("date".to_string(), Value::String(iso));
            }
        } else if let Some(v) = kw(trimmed, "UPDATED") {
            if let Some(iso) = crate::util::org_date_to_iso(v) {
                self.frontmatter
                    .insert("updated".to_string(), Value::String(iso));
            }
        } else if let Some(v) = kw(trimmed, "ABBRLINK") {
            self.frontmatter
                .insert("abbrlink".to_string(), Value::String(v.to_string()));
        } else if let Some(v) = kw(trimmed, "TAGS") {
            let items: Vec<Value> = v
                .split(',')
                .map(|s| Value::String(s.trim().to_string()))
                .filter(|v| !v.as_str().unwrap_or("").is_empty())
                .collect();
            self.frontmatter
                .insert("tags".to_string(), Value::Array(items));
        } else if let Some(v) = kw(trimmed, "CATEGORY") {
            self.frontmatter.insert(
                "category".to_string(),
                Value::Array(vec![Value::String(v.to_string())]),
            );
        } else if let Some(v) = kw(trimmed, "AUTHOR") {
            self.frontmatter
                .insert("author".to_string(), Value::String(v.to_string()));
        } else if let Some(v) = kw(trimmed, "EMAIL") {
            self.frontmatter
                .insert("email".to_string(), Value::String(v.to_string()));
        } else if let Some(v) = kw(trimmed, "ALIAS") {
            let list = self
                .frontmatter
                .entry("alias".to_string())
                .or_insert_with(|| Value::Array(vec![]));
            if let Value::Array(ref mut l) = list {
                l.push(Value::String(v.to_string()));
            }
        } else if let Some(v) = kw(trimmed, "LANGUAGE") {
            self.frontmatter
                .insert("language".to_string(), Value::String(v.to_string()));
        } else if let Some(v) = kw(trimmed, "ATTR_HTML") {
            self.frontmatter
                .insert("attr_html".to_string(), Value::String(v.to_string()));
        } else if let Some(v) = kw(trimmed, "OPTIONS") {
            let entry = self
                .frontmatter
                .entry("org".to_string())
                .or_insert_with(|| {
                    let mut org_map = serde_json::Map::new();
                    org_map.insert("options".to_string(), Value::Object(serde_json::Map::new()));
                    Value::Object(org_map)
                });
            if let Value::Object(ref mut org_map) = entry {
                if let Some(Value::Object(ref mut opts)) = org_map.get_mut("options") {
                    let tokens: Vec<&str> = v.split_whitespace().collect();
                    let mut i = 0;
                    while i < tokens.len() {
                        if let Some((key, val)) = tokens[i].split_once(':') {
                            let mapped_key = match key {
                                "^" => "superscript",
                                other => other,
                            };
                            if val.is_empty() && i + 1 < tokens.len() {
                                let mapped_val = match tokens[i + 1] {
                                    "nil" => Value::Bool(false),
                                    "t" => Value::Bool(true),
                                    other => Value::String(other.to_string()),
                                };
                                opts.insert(mapped_key.to_string(), mapped_val);
                                i += 2;
                            } else {
                                let mapped_val = match val {
                                    "nil" => Value::Bool(false),
                                    "t" => Value::Bool(true),
                                    other => Value::String(other.to_string()),
                                };
                                opts.insert(mapped_key.to_string(), mapped_val);
                                i += 1;
                            }
                        } else {
                            i += 1;
                        }
                    }
                }
            }
        } else if let Some(v) = kw(trimmed, "LINK") {
            if let Some(sp) = v.find(char::is_whitespace) {
                let name = v[..sp].trim().to_string();
                let url = v[sp..].trim().to_string();
                self.link_aliases.insert(name, url);
            }
        }
    }

    fn parse_blocks(&mut self) -> Result<Vec<Node>> {
        let mut blocks = Vec::new();
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos].clone();
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if blocks.is_empty() {
                    self.advance();
                    continue;
                }
                blocks.push(Node::new("blankLine"));
                self.advance();
                continue;
            }
            if let Some((depth, title, tags)) = crate::parser::org::heading::parse_heading(&line) {
                if crate::parser::org::heading::should_skip_section(&tags) {
                    self.skip_section(depth);
                    continue;
                }
                let content = parse_inline(title, &self.link_aliases);
                let tags_vec: Vec<Value> = tags
                    .into_iter()
                    .map(|s| Value::String(s.to_string()))
                    .collect();
                let heading = Node::new("heading")
                    .with_children(content)
                    .data_num("depth", depth as u64)
                    .data_list_val("tags", tags_vec);
                blocks.push(heading);
                self.advance();
                continue;
            }
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
            if let Some(v) = kw(trimmed, "JSX") {
                blocks.push(Node::new("html").with_value(v.trim()));
                self.advance();
                continue;
            }
            if let Some(v) = kw(trimmed, "HTML") {
                blocks.push(
                    Node::new("html").with_value(&crate::parser::html::html_to_jsx(v.trim())),
                );
                self.advance();
                continue;
            }
            if trimmed.starts_with("#+") || trimmed.starts_with("#-") {
                self.advance();
                continue;
            }
            if line.starts_with(": ") || line == ":" {
                let code = self.parse_colon_block();
                blocks.push(Node::new("code").with_value(&code));
                continue;
            }
            if crate::parser::org::list::is_unordered_item(&line) {
                let list = self.parse_unordered_list()?;
                blocks.push(list);
                continue;
            }
            if crate::parser::org::list::is_ordered_item(&line) {
                let list = self.parse_ordered_list()?;
                blocks.push(list);
                continue;
            }
            let (para, _had_lb) = self.parse_paragraph();
            if let Some(ref children) = para.children {
                if !children.is_empty() {
                    blocks.push(para);
                }
            }
        }
        Ok(blocks)
    }

    fn skip_section(&mut self, heading_level: u32) {
        self.advance();
        while let Some(line) = self.peek() {
            if let Some((lvl, _, _)) = crate::parser::org::heading::parse_heading(line) {
                if lvl <= heading_level {
                    break;
                }
            }
            self.advance();
        }
    }

    fn parse_block(&mut self, start_line: &str) -> Result<Node> {
        let block_type = crate::parser::org::block::parse_block_begin(start_line);
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
            crate::parser::org::block::BlockType::Src(opts) => {
                if opts.exports.as_deref() == Some("none") {
                    return Ok(Node::new("comment").with_value(":exports none"));
                }
                let min_indent = lines
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.len() - l.trim_start_matches([' ', '\t']).len())
                    .min()
                    .unwrap_or(0);
                let stripped: Vec<String> = lines
                    .iter()
                    .map(|l| crate::util::strip_prefix_spaces(l, min_indent).to_string())
                    .collect();
                let content = stripped.join("\n");
                Ok(Node::new("code")
                    .with_value(&content)
                    .data_str("lang", &opts.lang))
            }
            crate::parser::org::block::BlockType::Example => {
                let min_indent = lines
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.len() - l.trim_start_matches([' ', '\t']).len())
                    .min()
                    .unwrap_or(0);
                let stripped: Vec<String> = lines
                    .iter()
                    .map(|l| crate::util::strip_prefix_spaces(l, min_indent).to_string())
                    .collect();
                let content = stripped.join("\n");
                Ok(Node::new("code")
                    .with_value(&content)
                    .data_str("block_type", "example"))
            }
            crate::parser::org::block::BlockType::Quote => {
                let mut quote_blocks = Vec::new();
                for line in lines {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        quote_blocks.push(Node::new("blankLine"));
                    } else {
                        let para = Node::new("paragraph")
                            .with_children(parse_inline(trimmed, &self.link_aliases));
                        quote_blocks.push(para);
                    }
                }
                Ok(Node::new("blockquote").with_children(quote_blocks))
            }
            crate::parser::org::block::BlockType::Center => {
                let content = lines.join(" ");
                Ok(
                    Node::new("paragraph")
                        .with_children(parse_inline(&content, &self.link_aliases)),
                )
            }
            crate::parser::org::block::BlockType::Export(opts) => {
                if opts.exports.as_deref() == Some("none") {
                    return Ok(Node::new("comment").with_value(":exports none"));
                }
                let content = lines.join("\n");
                Ok(Node::new("export")
                    .with_value(&content)
                    .data_str("lang", &opts.export_type))
            }
            crate::parser::org::block::BlockType::Unknown(_) => {
                let content = lines.join(" ");
                Ok(
                    Node::new("paragraph")
                        .with_children(parse_inline(&content, &self.link_aliases)),
                )
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

    fn parse_paragraph(&mut self) -> (Node, bool) {
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
            if crate::parser::org::heading::parse_heading(line).is_some() {
                break;
            }
            if crate::parser::org::list::is_unordered_item(line)
                || crate::parser::org::list::is_ordered_item(line)
            {
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
        let para = if had_line_break {
            Node::new("paragraph")
                .with_children(content)
                .data_bool("hardLineBreak", true)
        } else {
            Node::new("paragraph").with_children(content)
        };
        (para, had_line_break)
    }

    fn parse_unordered_list(&mut self) -> Result<Node> {
        let mut items = Vec::new();
        while let Some(line) = self.peek() {
            if !crate::parser::org::list::is_unordered_item(line) {
                break;
            }
            let content_raw = crate::parser::org::list::unordered_content(line);
            let content_inline = parse_inline(content_raw.trim(), &self.link_aliases);
            let mut item_blocks = vec![Node::new("paragraph").with_children(content_inline)];
            self.advance();
            let nested = self.parse_list_nested()?;
            if let Some(ref nc) = nested.children {
                if !nc.is_empty() {
                    item_blocks.push(nested);
                }
            }
            items.push(Node::new("listItem").with_children(item_blocks));
        }
        Ok(Node::new("list")
            .with_children(items)
            .data_bool("ordered", false))
    }

    fn parse_ordered_list(&mut self) -> Result<Node> {
        let mut items = Vec::new();
        while let Some(line) = self.peek() {
            if !crate::parser::org::list::is_ordered_item(line) {
                break;
            }
            let (_, content_raw) = crate::parser::org::list::ordered_parts(line);
            let content_inline = parse_inline(content_raw.trim(), &self.link_aliases);
            let mut item_blocks = vec![Node::new("paragraph").with_children(content_inline)];
            self.advance();
            let nested = self.parse_list_nested()?;
            if let Some(ref nc) = nested.children {
                if !nc.is_empty() {
                    item_blocks.push(nested);
                }
            }
            items.push(Node::new("listItem").with_children(item_blocks));
        }
        Ok(Node::new("list")
            .with_children(items)
            .data_bool("ordered", true))
    }

    fn parse_list_nested(&mut self) -> Result<Node> {
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
            if crate::parser::org::list::is_unordered_item(trimmed) {
                let content_raw = crate::parser::org::list::unordered_content(trimmed);
                let content_inline = parse_inline(content_raw.trim(), &self.link_aliases);
                items.push(
                    Node::new("listItem")
                        .with_children(vec![Node::new("paragraph").with_children(content_inline)]),
                );
                self.advance();
            } else if crate::parser::org::list::is_ordered_item(trimmed) {
                let (_, content_raw) = crate::parser::org::list::ordered_parts(trimmed);
                let content_inline = parse_inline(content_raw.trim(), &self.link_aliases);
                items.push(
                    Node::new("listItem")
                        .with_children(vec![Node::new("paragraph").with_children(content_inline)]),
                );
                self.advance();
            } else {
                break;
            }
        }
        Ok(Node::new("list")
            .with_children(items)
            .data_bool("ordered", false))
    }
}

pub fn parse_org(input: &str) -> Result<Node> {
    OrgParser::new(input).parse()
}
