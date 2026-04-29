use crate::block::{parse_block_begin, render_block, BlockType};
use crate::error::{Error, Result};
use crate::frontmatter::FrontmatterBuilder;
use crate::heading::should_skip_section;
use crate::inline::convert_inline;
use crate::list::ListParser;
use crate::paragraph::ParagraphParser;
use crate::render::{push_block_exact, push_block_n};
use crate::util::{html_to_jsx, kw, org_date_to_iso};
use std::collections::HashMap;

pub struct OrgConverter {
    pub(crate) lines: Vec<String>,
    pub(crate) pos: usize,
    link_aliases: HashMap<String, String>,
    frontmatter: FrontmatterBuilder,
}

impl OrgConverter {
    pub fn new(input: &str) -> Self {
        OrgConverter {
            lines: input.lines().map(|l| l.to_string()).collect(),
            pos: 0,
            link_aliases: HashMap::new(),
            frontmatter: FrontmatterBuilder::new(),
        }
    }

    pub(crate) fn peek(&self) -> Option<&str> {
        self.lines.get(self.pos).map(|s| s.as_str())
    }

    pub(crate) fn advance(&mut self) -> Option<&str> {
        let l = self.lines.get(self.pos).map(|s| s.as_str());
        if l.is_some() {
            self.pos += 1;
        }
        l
    }

    pub fn run(&mut self) -> Result<String> {
        // Pass 1: collect directives
        for line in &self.lines.clone() {
            self.process_directive_line(line);
        }
        self.pos = 0;

        // Pass 2: build body
        let body = self.build_body()?;
        let fm = self.frontmatter.build();
        let combined = format!("{}{}", fm, body);
        let trimmed = combined.trim_end();
        let last_line = trimmed.lines().last().unwrap_or("");
        let needs_newline =
            last_line == "```" || last_line.starts_with('<') || last_line.starts_with('{');
        if needs_newline {
            Ok(format!("{}\n", trimmed))
        } else {
            Ok(trimmed.to_string())
        }
    }

    fn process_directive_line(&mut self, line: &str) {
        if let Some(v) = kw(line, "TITLE") {
            self.frontmatter.set_str("title", v.to_string());
        } else if let Some(v) = kw(line, "DATE") {
            if let Some(iso) = org_date_to_iso(v) {
                self.frontmatter.set_str("date", iso);
            }
        } else if let Some(v) = kw(line, "UPDATED") {
            if let Some(iso) = org_date_to_iso(v) {
                self.frontmatter.set_str("updated", iso);
            }
        } else if let Some(v) = kw(line, "ABBRLINK") {
            self.frontmatter.set_str("abbrlink", v.to_string());
        } else if let Some(v) = kw(line, "TAGS") {
            let items: Vec<String> = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            self.frontmatter.set_list("tags", items);
        } else if let Some(v) = kw(line, "CATEGORY") {
            self.frontmatter.set_list("category", vec![v.to_string()]);
        } else if let Some(v) = kw(line, "AUTHOR") {
            self.frontmatter.set_str("author", v.to_string());
        } else if let Some(v) = kw(line, "EMAIL") {
            self.frontmatter.set_str("author_email", v.to_string());
        } else if let Some(v) = kw(line, "ALIAS") {
            self.frontmatter.push_list("alias", v.to_string());
        } else if let Some(v) = kw(line, "LANGUAGE") {
            self.frontmatter.set_str("language", v.to_string());
        } else if let Some(v) = kw(line, "ATTR_HTML") {
            self.frontmatter.set_str("attr_html", v.to_string());
        } else if let Some(v) = kw(line, "LINK") {
            if let Some(sp) = v.find(char::is_whitespace) {
                let name = v[..sp].trim().to_string();
                let url = v[sp..].trim().to_string();
                self.link_aliases.insert(name, url);
            }
        }
    }

    fn build_body(&mut self) -> Result<String> {
        let mut out = String::new();
        let mut after_code = false;
        let mut after_html = false;
        let mut code_no_blank = false;

        // Skip leading blank lines
        while let Some(line) = self.peek() {
            if line.trim().is_empty() {
                self.advance();
            } else {
                break;
            }
        }

        while let Some(line_raw) = self.peek() {
            let line = line_raw.to_string();
            let trimmed_line = line.trim();

            if trimmed_line.is_empty() {
                let trailing = out.bytes().rev().take_while(|&b| b == b'\n').count();
                if !out.is_empty() && trailing < 2 {
                    out.push('\n');
                }
                self.advance();
                continue;
            }

            if trimmed_line.starts_with("#+") {
                if let Some(v) = kw(trimmed_line, "JSX") {
                    let jsx = v.trim().to_string();
                    let min_blanks = if after_code { 2 } else { 1 };
                    push_block_n(&mut out, &jsx, min_blanks);
                    after_code = false;
                    self.advance();
                    continue;
                }
                if let Some(v) = kw(trimmed_line, "HTML") {
                    let jsx = html_to_jsx(v.trim());
                    if code_no_blank {
                        if !out.ends_with('\n') {
                            out.push('\n');
                        }
                        out.push(' ');
                        out.push_str(&jsx);
                        out.push('\n');
                        code_no_blank = false;
                    } else {
                        let min_blanks = if after_code { 2 } else { 1 };
                        push_block_n(&mut out, &jsx, min_blanks);
                    }
                    after_code = true;
                    if v.trim().contains("more") {
                        after_html = true;
                    }
                    self.advance();
                    let next_is_blank = self.peek().map(|l| l.trim().is_empty()).unwrap_or(false);
                    if !next_is_blank {
                        code_no_blank = true;
                    }
                    continue;
                }
                let tl_lower = trimmed_line.to_lowercase();
                let block_indented = line != trimmed_line;
                if tl_lower.starts_with("#+begin_src")
                    || tl_lower.starts_with("#+begin_example")
                    || tl_lower.starts_with("#+begin_quote")
                    || tl_lower.starts_with("#+begin_center")
                    || tl_lower.starts_with("#+begin_export")
                    || tl_lower.starts_with("#+begin_")
                {
                    let block_type = parse_block_begin(trimmed_line);
                    self.advance();
                    let end_kw = block_type.end_keyword();
                    let mut block_lines = Vec::new();
                    loop {
                        match self.peek() {
                            None => break,
                            Some(bl) => {
                                if bl
                                    .trim()
                                    .to_lowercase()
                                    .starts_with(&format!("#+end_{}", end_kw))
                                {
                                    self.advance();
                                    break;
                                }
                                block_lines.push(bl.to_string());
                                self.advance();
                            }
                        }
                    }
                    match &block_type {
                        BlockType::Export => {
                            after_code = false;
                        }
                        BlockType::Unknown(_) => {
                            let rendered =
                                render_block(&block_type, &block_lines, |s| self.inline(s));
                            if !rendered.is_empty() {
                                if code_no_blank {
                                    if !out.ends_with('\n') {
                                        out.push('\n');
                                    }
                                    out.push(' ');
                                    out.push_str(&rendered);
                                    out.push('\n');
                                    code_no_blank = false;
                                } else {
                                    let min_blanks = if after_code { 2 } else { 1 };
                                    push_block_n(&mut out, &rendered, min_blanks);
                                }
                                after_code = false;
                            }
                        }
                        _ => {
                            let rendered =
                                render_block(&block_type, &block_lines, |s| self.inline(s));
                            let rendered = if block_indented && rendered.ends_with("```") {
                                let end_idx = rendered.rfind("```").ok_or_else(|| {
                                    Error::InvalidOrgFile("Malformed code block".into())
                                })?;
                                format!("{}\n{}", &rendered[..end_idx], &rendered[end_idx..])
                            } else {
                                rendered
                            };
                            let min_blanks = match &block_type {
                                BlockType::Example => 2,
                                _ => {
                                    if after_code {
                                        2
                                    } else {
                                        1
                                    }
                                }
                            };
                            push_block_n(&mut out, &rendered, min_blanks);
                            match &block_type {
                                BlockType::Src(_) | BlockType::Example => {
                                    let next_is_blank =
                                        self.peek().map(|l| l.trim().is_empty()).unwrap_or(false);
                                    if !next_is_blank {
                                        code_no_blank = true;
                                    }
                                    after_code = false;
                                }
                                _ => {
                                    after_code = false;
                                }
                            }
                        }
                    }
                    continue;
                }
                self.advance();
                continue;
            }

            if let Some((level, title, tags)) = crate::heading::parse_heading(&line) {
                if should_skip_section(&tags) {
                    self.advance();
                    let heading_level = level;
                    loop {
                        match self.peek() {
                            None => break,
                            Some(l) => {
                                if let Some((lvl, _, _)) = crate::heading::parse_heading(l) {
                                    if lvl <= heading_level {
                                        break;
                                    }
                                }
                                self.advance();
                            }
                        }
                    }
                    continue;
                }
                let hashes = "#".repeat(level as usize);
                let heading = format!("{} {}", hashes, self.inline(title));
                let min_blanks = if after_html || after_code { 2 } else { 1 };
                push_block_exact(&mut out, &heading, min_blanks);
                after_code = false;
                after_html = false;
                code_no_blank = false;
                self.advance();
                continue;
            }

            if line.starts_with(": ") || line == ":" {
                let mut code_lines = Vec::new();
                while let Some(l) = self.peek() {
                    if l.starts_with(": ") {
                        code_lines.push(l[2..].to_string());
                        self.advance();
                    } else if l == ":" {
                        code_lines.push(String::new());
                        self.advance();
                    } else {
                        break;
                    }
                }
                let content = code_lines.join("\n");
                let rendered = format!("```\n{}\n```", content);
                let min_blanks = if after_code { 2 } else { 1 };
                push_block_n(&mut out, &rendered, min_blanks);
                continue;
            }

            if crate::list::is_unordered_item(&line) {
                let (list, had_sub) = self.collect_unordered_list()?;
                let min_blanks = if after_code { 2 } else { 1 };
                push_block_n(&mut out, &list, min_blanks);
                after_code = had_sub;
                continue;
            }

            if crate::list::is_ordered_item(&line) {
                let list = self.collect_ordered_list()?;
                push_block_n(&mut out, &list, 1);
                after_code = false;
                continue;
            }

            let (para, had_lb) = self.collect_paragraph();
            if !para.is_empty() {
                if code_no_blank {
                    let para_line = format!(" {}", para);
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str(&para_line);
                    out.push('\n');
                    code_no_blank = false;
                } else {
                    let min_blanks = if after_code || after_html { 2 } else { 1 };
                    push_block_n(&mut out, &para, min_blanks);
                    code_no_blank = false;
                }
                after_code = had_lb;
                after_html = false;
            }
        }
        Ok(out)
    }

    pub(crate) fn inline(&self, text: &str) -> String {
        convert_inline(text, &self.link_aliases)
    }
}

pub fn convert(input: &str) -> Result<String> {
    let mut conv = OrgConverter::new(input);
    conv.run()
}
