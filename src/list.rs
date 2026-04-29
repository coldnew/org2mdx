use crate::block::{parse_block_begin, render_block};
use crate::converter::OrgConverter;
use crate::error::{Error, Result};
// // use crate::inline::convert_inline;

pub trait ListParser {
    fn collect_unordered_list(&mut self) -> Result<(String, bool)>;
    fn collect_ordered_list(&mut self) -> Result<String>;
    fn collect_list_sub_content(
        &mut self,
        result: &mut String,
        had_sub_content: &mut bool,
    ) -> Result<()>;
}

impl ListParser for OrgConverter {
    fn collect_unordered_list(&mut self) -> Result<(String, bool)> {
        let mut result = String::new();
        let mut had_sub_content = false;
        let mut had_any_sub_content = false;
        loop {
            match self.peek() {
                None => break,
                Some(l) => {
                    let l = l.to_string();
                    if l.trim().is_empty() {
                        let next_non_blank = self.lines[self.pos + 1..]
                            .iter()
                            .find(|nl| !nl.trim().is_empty())
                            .map(|nl| nl.as_str());
                        match next_non_blank {
                            Some(next) if is_unordered_item(next) || is_ordered_item(next) => {
                                self.advance();
                                continue;
                            }
                            Some(next) if next.starts_with("  ") => {
                                self.advance();
                                self.collect_list_sub_content(&mut result, &mut had_sub_content)?;
                                if had_sub_content {
                                    had_any_sub_content = true;
                                }
                                continue;
                            }
                            _ => break,
                        }
                    } else if is_unordered_item(&l) {
                        if !result.is_empty() {
                            if had_sub_content {
                                result.push_str("\n\n");
                                had_sub_content = false;
                            } else {
                                result.push('\n');
                            }
                        }
                        let content = unordered_content(&l);
                        result.push_str(&format!("* {}", self.inline(content.trim_end())));
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }
        Ok((result, had_any_sub_content))
    }

    fn collect_ordered_list(&mut self) -> Result<String> {
        let mut result = String::new();
        loop {
            match self.peek() {
                None => break,
                Some(l) => {
                    let l = l.to_string();
                    if is_ordered_item(&l) {
                        let (num, content) = ordered_parts(&l);
                        if !result.is_empty() {
                            result.push('\n');
                        }
                        result += &format!("{}. {}", num, self.inline(content.trim()));
                        self.advance();
                    } else if l.trim().is_empty() {
                        let next_non_blank = self.lines[self.pos + 1..]
                            .iter()
                            .find(|nl| !nl.trim().is_empty())
                            .map(|nl| nl.as_str());
                        match next_non_blank {
                            Some(next) if is_ordered_item(next) => {
                                if result.contains("\n   ") {
                                    result.push('\n');
                                }
                                self.advance();
                                continue;
                            }
                            Some(next)
                                if next.starts_with("   ")
                                    && !is_unordered_item(next)
                                    && !next.trim_start().starts_with('#') =>
                            {
                                result.push('\n');
                                self.advance();
                                loop {
                                    match self.peek() {
                                        None => break,
                                        Some(inner) => {
                                            let inner = inner.to_string();
                                            if inner.trim().is_empty() {
                                                let nxt = self.lines[self.pos + 1..]
                                                    .iter()
                                                    .find(|nl| !nl.trim().is_empty())
                                                    .map(|nl| nl.as_str());
                                                match nxt {
                                                    Some(n)
                                                        if n.starts_with("   ")
                                                            && !is_ordered_item(n)
                                                            && !is_unordered_item(n)
                                                            && !n.trim_start().starts_with('#') =>
                                                    {
                                                        result.push('\n');
                                                        self.advance();
                                                        continue;
                                                    }
                                                    Some(n) if is_ordered_item(n) => break,
                                                    _ => break,
                                                }
                                            } else if inner.starts_with("   ")
                                                && !is_ordered_item(&inner)
                                                && !is_unordered_item(&inner)
                                                && !inner.trim_start().starts_with('#')
                                            {
                                                result +=
                                                    &format!("\n   {}", self.inline(inner.trim()));
                                                self.advance();
                                            } else {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(result)
    }

    fn collect_list_sub_content(
        &mut self,
        result: &mut String,
        had_sub_content: &mut bool,
    ) -> Result<()> {
        loop {
            match self.peek() {
                None => break,
                Some(l) => {
                    let l = l.to_string();
                    let lt = l.trim();
                    if lt.is_empty() {
                        let nxt = self.lines[self.pos + 1..]
                            .iter()
                            .find(|nl| !nl.trim().is_empty())
                            .map(|nl| nl.as_str());
                        match nxt {
                            Some(n) if n.starts_with("  ") => {
                                self.advance();
                                continue;
                            }
                            _ => break,
                        }
                    } else if l.starts_with("  ") {
                        let lt_lower = lt.to_lowercase();
                        if lt_lower.starts_with("#+begin_src")
                            || lt_lower.starts_with("#+begin_example")
                        {
                            let is_example = lt_lower.starts_with("#+begin_example");
                            let placeholder = if is_example { "  " } else { " " };
                            result.push('\n');
                            result.push('\n');
                            result.push_str(placeholder);
                            result.push('\n');
                            if !is_example {
                                result.push('\n');
                            }
                            let block_type = parse_block_begin(lt);
                            self.advance();
                            let mut block_lines = Vec::new();
                            loop {
                                match self.peek() {
                                    None => break,
                                    Some(bl) => {
                                        let blt = bl.trim().to_lowercase();
                                        if blt.starts_with("#+end_src")
                                            || blt.starts_with("#+end_example")
                                        {
                                            self.advance();
                                            break;
                                        }
                                        block_lines.push(bl.to_string());
                                        self.advance();
                                    }
                                }
                            }
                            let rendered =
                                render_block(&block_type, &block_lines, |s| self.inline(s));
                            let rendered_with_trail = if rendered.ends_with("```") {
                                let end_idx = rendered.rfind("```").ok_or_else(|| {
                                    Error::InvalidOrgFile("Malformed code block".into())
                                })?;
                                format!("{}\n{}", &rendered[..end_idx], &rendered[end_idx..])
                            } else {
                                rendered
                            };
                            result.push_str(&rendered_with_trail);
                            result.push('\n');
                            *had_sub_content = true;
                        } else if lt.starts_with('#') {
                            self.advance();
                        } else {
                            let mut sub_parts = vec![self.inline(lt)];
                            self.advance();
                            loop {
                                match self.peek() {
                                    Some(inner)
                                        if inner.starts_with("  ")
                                            && !inner.trim().is_empty()
                                            && !inner.trim().to_lowercase().starts_with("#+") =>
                                    {
                                        sub_parts.push(self.inline(inner.trim()));
                                        self.advance();
                                    }
                                    _ => break,
                                }
                            }
                            result.push('\n');
                            result.push('\n');
                            result.push_str(&format!("  {}", sub_parts.join(" ")));
                            *had_sub_content = true;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn is_unordered_item(line: &str) -> bool {
    let t = line.trim_start();
    (t.starts_with("- ") || t.starts_with("+ ")) && t.len() > 2
}

pub fn unordered_content(line: &str) -> &str {
    let t = line.trim_start();
    &t[2..]
}

pub fn is_ordered_item(line: &str) -> bool {
    let t = line.trim_start();
    let b = t.as_bytes();
    let mut i = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && b.get(i) == Some(&b'.') && b.get(i + 1) == Some(&b' ')
}

pub fn ordered_parts(line: &str) -> (&str, &str) {
    let t = line.trim_start();
    if let Some(dot) = t.find(". ") {
        (&t[..dot], &t[dot + 2..])
    } else {
        ("1", t)
    }
}
