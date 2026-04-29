use std::collections::HashMap;

/// Convert an org-mode file content to MDX format.
pub fn convert(input: &str) -> String {
    let mut conv = OrgConverter::new(input);
    conv.run()
}

enum FmVal {
    Str(String),
    List(Vec<String>),
}

struct OrgConverter {
    lines: Vec<String>,
    pos: usize,
    link_aliases: HashMap<String, String>,
    // frontmatter in insertion order
    fm_keys: Vec<String>,
    fm_vals: HashMap<String, FmVal>,
}

impl OrgConverter {
    fn new(input: &str) -> Self {
        OrgConverter {
            lines: input.lines().map(|l| l.to_string()).collect(),
            pos: 0,
            link_aliases: HashMap::new(),
            fm_keys: vec![],
            fm_vals: HashMap::new(),
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

    fn run(&mut self) -> String {
        // Pass 1: collect directives
        for line in &self.lines.clone() {
            self.process_directive_line(line);
        }
        self.pos = 0;

        // Pass 2: build body
        let body = self.build_body();
        let fm = self.build_frontmatter();
        let combined = format!("{}\n{}", fm, body);
        let trimmed = combined.trim_end();
        // Add trailing newline if last content is a code block or HTML element
        let last_line = trimmed.lines().last().unwrap_or("");
        let needs_newline =
            last_line == "```" || last_line.starts_with('<') || last_line.starts_with('{');
        if needs_newline {
            format!("{}\n", trimmed)
        } else {
            trimmed.to_string()
        }
    }

    fn fm_set_str(&mut self, key: &str, val: String) {
        if !self.fm_vals.contains_key(key) {
            self.fm_keys.push(key.to_string());
        }
        self.fm_vals.insert(key.to_string(), FmVal::Str(val));
    }

    fn fm_push_list(&mut self, key: &str, val: String) {
        if let Some(FmVal::List(ref mut v)) = self.fm_vals.get_mut(key) {
            v.push(val);
        } else {
            if !self.fm_vals.contains_key(key) {
                self.fm_keys.push(key.to_string());
            }
            self.fm_vals.insert(key.to_string(), FmVal::List(vec![val]));
        }
    }

    fn fm_set_list(&mut self, key: &str, vals: Vec<String>) {
        if !self.fm_vals.contains_key(key) {
            self.fm_keys.push(key.to_string());
        }
        self.fm_vals.insert(key.to_string(), FmVal::List(vals));
    }

    fn process_directive_line(&mut self, line: &str) {
        if let Some(v) = kw(line, "TITLE") {
            self.fm_set_str("title", v.to_string());
        } else if let Some(v) = kw(line, "DATE") {
            if let Some(iso) = org_date_to_iso(v) {
                self.fm_set_str("date", iso);
            }
        } else if let Some(v) = kw(line, "UPDATED") {
            if let Some(iso) = org_date_to_iso(v) {
                self.fm_set_str("updated", iso);
            }
        } else if let Some(v) = kw(line, "ABBRLINK") {
            self.fm_set_str("abbrlink", v.to_string());
        } else if let Some(v) = kw(line, "TAGS") {
            let items: Vec<String> = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            self.fm_set_list("tags", items);
        } else if let Some(v) = kw(line, "CATEGORY") {
            self.fm_set_list("category", vec![v.to_string()]);
        } else if let Some(v) = kw(line, "AUTHOR") {
            self.fm_set_str("author", v.to_string());
        } else if let Some(v) = kw(line, "EMAIL") {
            self.fm_set_str("author_email", v.to_string());
        } else if let Some(v) = kw(line, "ALIAS") {
            self.fm_push_list("alias", v.to_string());
        } else if let Some(v) = kw(line, "ATTR_HTML") {
            self.fm_set_str("attr_html", v.to_string());
        } else if let Some(v) = kw(line, "LINK") {
            if let Some(sp) = v.find(char::is_whitespace) {
                let name = v[..sp].trim().to_string();
                let url = v[sp..].trim().to_string();
                self.link_aliases.insert(name, url);
            }
        }
    }

    fn build_frontmatter(&self) -> String {
        let mut s = String::from("---\n");
        for key in &self.fm_keys {
            match self.fm_vals.get(key) {
                Some(FmVal::Str(v)) => {
                    if key == "abbrlink" {
                        s += &format!("{}: {}\n", key, v);
                    } else {
                        s += &format!("{}: {}\n", key, yaml_str(v));
                    }
                }
                Some(FmVal::List(items)) => {
                    s += &format!("{}:\n", key);
                    for item in items {
                        s += &format!("  - {}\n", item);
                    }
                }
                None => {}
            }
        }
        s += "description: Generated from Org-mode\n";
        s += "---";
        s
    }

    fn build_body(&mut self) -> String {
        let mut out = String::new();
        let mut after_code = false;
        let mut after_html = false;
        let mut code_no_blank = false;

        while let Some(line_raw) = self.peek() {
            let line = line_raw.to_string();
            let trimmed_line = line.trim();

            // Skip blank lines (but track org blank lines as separators)
            if trimmed_line.is_empty() {
                if !out.is_empty() && (after_code || !out.ends_with("\n\n")) {
                    out.push('\n');
                }
                self.advance();
                continue;
            }

            // Skip remaining directives (#+KEYWORD:)
            if trimmed_line.starts_with("#+") {
                // Check for JSX
                if let Some(v) = kw(trimmed_line, "JSX") {
                    let jsx = v.trim().to_string();
                    let min_blanks = if after_code { 2 } else { 1 };
                    push_block_n(&mut out, &jsx, min_blanks);
                    after_code = false;
                    self.advance();
                    continue;
                }
                // Check for HTML
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
                // code/example blocks
                let tl_lower = trimmed_line.to_lowercase();
                let block_indented = line != trimmed_line; // original line was indented
                if tl_lower.starts_with("#+begin_src")
                    || tl_lower.starts_with("#+begin_example")
                    || tl_lower.starts_with("#+begin_quote")
                    || tl_lower.starts_with("#+begin_center")
                    || tl_lower.starts_with("#+begin_export")
                    || tl_lower.starts_with("#+begin_")
                {
                    let block_type = parse_block_begin(trimmed_line);
                    self.advance(); // consume #+begin_* line
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
                            // skip content entirely
                            after_code = false;
                        }
                        BlockType::Unknown(_) => {
                            let rendered = self.render_block(&block_type, &block_lines);
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
                            let rendered = self.render_block(&block_type, &block_lines);
                            // Indented code blocks get a trailing blank inside the fence
                            let rendered = if block_indented && rendered.ends_with("```") {
                                let end_idx = rendered.rfind("```").unwrap();
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
                            // Only Src and Example blocks trigger after_code
                            match &block_type {
                                BlockType::Src(_) | BlockType::Example => {
                                    let next_is_blank =
                                        self.peek().map(|l| l.trim().is_empty()).unwrap_or(false);
                                    if next_is_blank {
                                        out.push('\n'); // extra blank after code block
                                    } else {
                                        code_no_blank = true;
                                    }
                                    after_code = true;
                                }
                                _ => {
                                    after_code = false;
                                }
                            }
                        }
                    }
                    continue;
                }
                // other #+KEYWORD: skip
                self.advance();
                continue;
            }

            // Heading
            if let Some((level, title, tags)) = parse_heading(&line) {
                if tags.iter().any(|t| *t == "noexport") {
                    // skip this entire section
                    self.advance();
                    let heading_level = level;
                    loop {
                        match self.peek() {
                            None => break,
                            Some(l) => {
                                if let Some((lvl, _, _)) = parse_heading(l) {
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
                // For headings, enforce exact blank count (cap excess)
                push_block_exact(&mut out, &heading, min_blanks);
                after_code = false;
                after_html = false;
                code_no_blank = false;
                self.advance();
                continue;
            }

            // Colon-prefix example lines
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
                // Colon-example blocks do NOT add extra blank or set after_code
                continue;
            }

            // Unordered list
            if is_unordered_item(&line) {
                let (list, had_sub) = self.collect_unordered_list();
                let min_blanks = if after_code { 2 } else { 1 };
                push_block_n(&mut out, &list, min_blanks);
                after_code = if had_sub { true } else { false };
                continue;
            }

            // Ordered list
            if is_ordered_item(&line) {
                let list = self.collect_ordered_list();
                push_block_n(&mut out, &list, 1);
                after_code = false;
                continue;
            }

            // Regular paragraph
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
        out
    }

    fn render_block(&self, bt: &BlockType, lines: &[String]) -> String {
        // Strip only leading blank lines from block content (preserve trailing blanks)
        let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
        let lines = &lines[start..];
        // Calculate minimum indentation (count spaces and tabs)
        let min_indent = lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start_matches(|c| c == ' ' || c == '\t').len())
            .min()
            .unwrap_or(0);
        match bt {
            BlockType::Src(lang) => {
                let mut s = format!("```{}\n", lang);
                for l in lines {
                    s += strip_prefix_spaces(l, min_indent);
                    s.push('\n');
                }
                s += "```";
                s
            }
            BlockType::Example => {
                let mut s = String::from("```\n");
                for l in lines {
                    s += strip_prefix_spaces(l, min_indent);
                    s.push('\n');
                }
                s += "```";
                s
            }
            BlockType::Quote => {
                let mut s = String::new();
                for (i, l) in lines.iter().enumerate() {
                    let t = strip_prefix_spaces(l, min_indent).trim();
                    if i > 0 {
                        s.push('\n');
                    }
                    if t.is_empty() {
                        s.push('>');
                    } else {
                        s.push_str(&format!("> {}", self.inline(t)));
                    }
                }
                s
            }
            BlockType::Center => {
                // center blocks: emit content as inline
                let parts: Vec<String> = lines
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| self.inline(l.trim()))
                    .collect();
                parts.join(" ")
            }
            BlockType::Export => String::new(),
            BlockType::Unknown(_) => {
                // emit as paragraph text
                let parts: Vec<String> = lines
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| self.inline(l.trim()))
                    .collect();
                parts.join(" ")
            }
        }
    }

    fn collect_unordered_list(&mut self) -> (String, bool) {
        let mut result = String::new();
        let mut had_sub_content = false;
        let mut had_any_sub_content = false;
        loop {
            match self.peek() {
                None => break,
                Some(l) => {
                    let l = l.to_string();
                    if l.trim().is_empty() {
                        // Look ahead: determine what comes after blank
                        let next_non_blank = self.lines[self.pos + 1..]
                            .iter()
                            .find(|nl| !nl.trim().is_empty())
                            .map(|nl| nl.as_str());
                        match next_non_blank {
                            Some(next) if is_unordered_item(next) || is_ordered_item(next) => {
                                // blank between items: skip blank and continue
                                self.advance();
                                continue;
                            }
                            Some(next) if next.starts_with("  ") => {
                                // blank before indented sub-content (para or code block)
                                self.advance(); // consume blank
                                                // Now collect sub-content: indented paras and/or indented code blocks
                                self.collect_list_sub_content(&mut result, &mut had_sub_content);
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
        (result, had_any_sub_content)
    }

    /// Collect sub-content after a list item: indented paragraphs and indented code blocks.
    /// Appends to `result`. Updates `had_sub_content`.
    fn collect_list_sub_content(&mut self, result: &mut String, had_sub_content: &mut bool) {
        loop {
            match self.peek() {
                None => break,
                Some(l) => {
                    let l = l.to_string();
                    let lt = l.trim();
                    if lt.is_empty() {
                        // Look ahead
                        let nxt = self.lines[self.pos + 1..]
                            .iter()
                            .find(|nl| !nl.trim().is_empty())
                            .map(|nl| nl.as_str());
                        match nxt {
                            Some(n) if n.starts_with("  ") => {
                                // more sub-content follows
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
                            // Indented code block inside list item
                            let is_example = lt_lower.starts_with("#+begin_example");
                            let placeholder = if is_example { "  " } else { " " };
                            // Add the space placeholder before the code block
                            result.push('\n');
                            result.push('\n');
                            result.push_str(placeholder);
                            result.push('\n');
                            if !is_example {
                                // src blocks: 1 blank between placeholder and code
                                result.push('\n');
                            }
                            // example blocks: no blank between placeholder and code
                            // Parse the block type and collect lines
                            let block_type = parse_block_begin(lt);
                            self.advance(); // consume #+begin_* line
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
                            let rendered = self.render_block(&block_type, &block_lines);
                            // For list sub-content code blocks, add trailing blank inside
                            // (original converter quirk: blank line before closing ```)
                            let rendered_with_trail = if rendered.ends_with("```") {
                                let end_idx = rendered.rfind("```").unwrap();
                                format!("{}\n{}", &rendered[..end_idx], &rendered[end_idx..])
                            } else {
                                rendered
                            };
                            result.push_str(&rendered_with_trail);
                            result.push('\n');
                            // had_sub_content will make the next item use \n\n separator
                            *had_sub_content = true;
                        } else if lt.starts_with('#') {
                            // Other #+KEYWORD: skip
                            self.advance();
                        } else {
                            // Indented paragraph line
                            let mut sub_parts = vec![self.inline(lt)];
                            self.advance();
                            // Collect continuation lines
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
    }

    fn collect_ordered_list(&mut self) -> String {
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
                        // Look ahead to determine what comes next
                        let next_non_blank = self.lines[self.pos + 1..]
                            .iter()
                            .find(|nl| !nl.trim().is_empty())
                            .map(|nl| nl.as_str());
                        match next_non_blank {
                            Some(next) if is_ordered_item(next) => {
                                // blank between items: if result ends with sub-content (indented), add blank
                                if result.contains("\n   ") {
                                    result.push('\n');
                                }
                                self.advance(); // consume the blank
                                continue;
                            }
                            Some(next)
                                if next.starts_with("   ")
                                    && !is_unordered_item(next)
                                    && !next.trim_start().starts_with('#') =>
                            {
                                // blank before indented continuation: emit blank, then consume continuation
                                result.push('\n');
                                self.advance(); // consume the blank
                                                // Now consume all consecutive indented lines (possibly separated by blanks within)
                                loop {
                                    match self.peek() {
                                        None => break,
                                        Some(inner) => {
                                            let inner = inner.to_string();
                                            if inner.trim().is_empty() {
                                                // check if next non-blank is still indented or another item
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
                                                    Some(n) if is_ordered_item(n) => {
                                                        // blank before next item - exit inner loop
                                                        break;
                                                    }
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
        result
    }

    fn collect_paragraph(&mut self) -> (String, bool) {
        let mut parts = vec![];
        let mut had_line_break = false;
        loop {
            match self.peek() {
                None => break,
                Some(l) => {
                    let l = l.to_string();
                    if l.trim().is_empty() {
                        break;
                    }
                    let lt = l.trim();
                    if lt.starts_with("#+") || lt.starts_with("#-") {
                        break;
                    }
                    if parse_heading(&l).is_some() {
                        break;
                    }
                    if is_unordered_item(&l) {
                        break;
                    }
                    if is_ordered_item(&l) {
                        break;
                    }
                    let trimmed = l.trim();
                    // Strip org line-break marker \\ at end of line
                    let (trimmed, is_lb) = if trimmed.ends_with("\\\\") {
                        (trimmed[..trimmed.len() - 2].trim_end(), true)
                    } else {
                        (trimmed, false)
                    };
                    if is_lb {
                        had_line_break = true;
                    }
                    parts.push(trimmed.to_string());
                    self.advance();
                }
            }
        }
        if parts.is_empty() {
            return (String::new(), false);
        }
        // Normalize multiple spaces to single (within each line and across join)
        let joined = parts.join(" ");
        // Collapse runs of 2+ spaces to single space, but preserve inside = and ~ spans
        let normalized = collapse_spaces(&joined);
        (self.inline(&normalized), had_line_break)
    }

    /// Convert inline org syntax to MDX
    fn inline(&self, text: &str) -> String {
        let mut out = String::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // [[...]] link
            if i + 1 < len && chars[i] == '[' && chars[i + 1] == '[' {
                if let Some((mdx, consumed)) = self.parse_link_at(&chars, i) {
                    out.push_str(&mdx);
                    i += consumed;
                    continue;
                }
            }

            // Bare URL: http:// or https://
            {
                let remaining: String = chars[i..].iter().collect();
                if remaining.starts_with("http://") || remaining.starts_with("https://") {
                    let url_start = i;
                    while i < len && !chars[i].is_whitespace() {
                        i += 1;
                    }
                    let url: String = chars[url_start..i].iter().collect();
                    let url_escaped = escape_url_parens(&url);
                    out.push_str(&format!("[{}]({})", url, url_escaped));
                    continue;
                }
            }

            // *bold*
            if chars[i] == '*' {
                if let Some((inner, n)) = markup_at(&chars, i, '*') {
                    out.push_str(&format!("**{}**", self.inline(&inner)));
                    i += n;
                    continue;
                }
            }

            // /italic/
            if chars[i] == '/' {
                if let Some((inner, n)) = markup_at(&chars, i, '/') {
                    out.push_str(&format!("*{}*", self.inline(&inner)));
                    i += n;
                    continue;
                }
            }

            // +strikethrough+
            if chars[i] == '+' {
                if let Some((inner, n)) = markup_at(&chars, i, '+') {
                    out.push_str(&format!("~~{}~~", self.inline(&inner)));
                    i += n;
                    continue;
                }
            }

            // =verbatim=
            if chars[i] == '=' {
                if let Some((inner, n)) = markup_at(&chars, i, '=') {
                    out.push_str(&format!("`{}`", inner));
                    i += n;
                    continue;
                }
            }

            // ~code~
            if chars[i] == '~' {
                if let Some((inner, n)) = markup_at(&chars, i, '~') {
                    out.push_str(&format!("`{}`", inner));
                    i += n;
                    continue;
                }
                // Bare ~ -> \~
                out.push_str("\\~");
                i += 1;
                continue;
            }

            // Subscript: word_subscript → wordsubscript (strip underscore, keep text)
            if chars[i] == '_' && i > 0 && chars[i - 1].is_alphanumeric() {
                if i + 1 < len {
                    let next = chars[i + 1];
                    if next.is_alphanumeric() {
                        // Skip the underscore (subscript stripping)
                        i += 1;
                        continue;
                    } else if next == '{' {
                        // _{...} subscript: skip underscore and braces, keep content
                        let mut j = i + 2;
                        while j < len && chars[j] != '}' {
                            out.push(chars[j]);
                            j += 1;
                        }
                        i = if j < len { j + 1 } else { j };
                        continue;
                    }
                }
            }

            out.push(chars[i]);
            i += 1;
        }

        out
    }

    fn parse_link_at(&self, chars: &[char], start: usize) -> Option<(String, usize)> {
        // chars[start] = '[', chars[start+1] = '['
        let mut i = start + 2;
        let mut target = String::new();
        let mut depth = 1;

        while i < chars.len() {
            match chars[i] {
                '[' => {
                    depth += 1;
                    target.push('[');
                }
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                    target.push(']');
                }
                c => target.push(c),
            }
            i += 1;
        }

        // Optional description: stop at first ]] (like original converter)
        let desc = if i < chars.len() && chars[i] == '[' {
            i += 1;
            let mut d = String::new();
            while i < chars.len() {
                // Stop at ]] (first double-close-bracket)
                if chars[i] == ']' && i + 1 < chars.len() && chars[i + 1] == ']' {
                    i += 2; // skip ]]
                    break;
                }
                match chars[i] {
                    '[' => d.push_str("\\["),
                    ']' => d.push_str("\\]"),
                    c => d.push(c),
                }
                i += 1;
            }
            Some(d)
        } else {
            None
        };

        // Optional final ] (only for no-desc links: [[url]])
        // For desc links, the ]] was already consumed inside desc parsing,
        // and any leftover ] should remain as literal text.
        if desc.is_none() {
            if i < chars.len() && chars[i] == ']' {
                i += 1;
            }
        }

        let consumed = i - start;

        // file: -> image (only for image extensions) or link (for other files)
        if let Some(path) = target.strip_prefix("file:") {
            let lower = path.to_lowercase();
            let is_image = lower.ends_with(".png")
                || lower.ends_with(".jpg")
                || lower.ends_with(".jpeg")
                || lower.ends_with(".gif")
                || lower.ends_with(".svg")
                || lower.ends_with(".webp");
            let encoded = pct_encode(path);
            if is_image && desc.is_none() {
                return Some((format!("![img]({})", encoded), consumed));
            } else {
                let display = desc.as_deref().unwrap_or(path).to_string();
                let display = self.inline(&display);
                // For non-image files, use file: URL as-is in link
                return Some((format!("[{}](file:{})", display, encoded), consumed));
            }
        }

        // Named alias
        if let Some(url) = self.link_aliases.get(&target) {
            let display = desc.as_deref().unwrap_or(&target).to_string();
            let display = self.inline(&display);
            let url = escape_url_parens(url);
            return Some((format!("[{}]({})", display, url), consumed));
        }

        // Bare URL (no description) -> [url](url)
        let url = escape_url_parens(&target);
        if let Some(d) = desc {
            let display = self.inline(&d);
            Some((format!("[{}]({})", display, url), consumed))
        } else {
            // No description: use raw target as display text (avoid double-processing URLs)
            // Escape & in display text too (MDX requires \& in link text)
            let display = target.replace('&', "\\&");
            Some((format!("[{}]({})", display, url), consumed))
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Block types

enum BlockType {
    Src(String),
    Example,
    Quote,
    Center,
    Export,
    Unknown(String),
}

impl BlockType {
    fn end_keyword(&self) -> String {
        match self {
            BlockType::Src(_) => "src".to_string(),
            BlockType::Example => "example".to_string(),
            BlockType::Quote => "quote".to_string(),
            BlockType::Center => "center".to_string(),
            BlockType::Export => "export".to_string(),
            BlockType::Unknown(name) => name.clone(),
        }
    }
}

fn parse_block_begin(line: &str) -> BlockType {
    let lower = line.trim().to_lowercase();
    if let Some(rest) = lower.strip_prefix("#+begin_src") {
        let lang = rest.trim().to_string();
        // Normalize language names
        let lang = match lang.as_str() {
            "c++" | "cpp" => "c".to_string(),
            other => other.to_string(),
        };
        BlockType::Src(lang)
    } else if lower.starts_with("#+begin_example") {
        BlockType::Example
    } else if lower.starts_with("#+begin_quote") {
        BlockType::Quote
    } else if lower.starts_with("#+begin_center") {
        BlockType::Center
    } else if lower.starts_with("#+begin_export") {
        BlockType::Export
    } else {
        // Unknown block type — extract block name
        let name = lower
            .strip_prefix("#+begin_")
            .unwrap_or("unknown")
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();
        BlockType::Unknown(name)
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Helpers

fn push_block(out: &mut String, content: &str) {
    push_block_n(out, content, 1);
}

fn push_block_n(out: &mut String, content: &str, min_blanks: usize) {
    if content.is_empty() {
        return;
    }
    let needed_newlines = min_blanks + 1; // blanks = newlines - 1 between blocks
    if out.is_empty() {
        out.push('\n'); // first block: start with single blank line
    } else {
        // Count trailing newlines currently in `out`
        let trailing = out.bytes().rev().take_while(|&b| b == b'\n').count();
        if trailing < needed_newlines {
            for _ in trailing..needed_newlines {
                out.push('\n');
            }
        }
    }
    out.push_str(content);
    if !out.ends_with('\n') {
        out.push('\n');
    }
}

/// Like push_block_n but also trims excess trailing newlines (exact blank count)
fn push_block_exact(out: &mut String, content: &str, blanks: usize) {
    if content.is_empty() {
        return;
    }
    let needed_newlines = blanks + 1;
    if out.is_empty() {
        out.push('\n');
    } else {
        let trailing = out.bytes().rev().take_while(|&b| b == b'\n').count();
        if trailing < needed_newlines {
            for _ in trailing..needed_newlines {
                out.push('\n');
            }
        } else if trailing > needed_newlines {
            let excess = trailing - needed_newlines;
            let new_len = out.len() - excess;
            out.truncate(new_len);
        }
    }
    out.push_str(content);
    if !out.ends_with('\n') {
        out.push('\n');
    }
}

/// Collapse multiple consecutive spaces to single space in plain text
/// (simple approach: replace "  " repeatedly)
fn collapse_spaces(s: &str) -> String {
    let mut result = s.to_string();
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    result
}

/// Get keyword value: "#+KEYWORD: value"  (case-insensitive keyword)
fn kw<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let prefix = format!("#+{}:", keyword.to_uppercase());
    let lp = format!("#+{}:", keyword.to_lowercase());
    let t = line.trim_start();
    let tu = t.to_uppercase();
    if tu.starts_with(&prefix) {
        let rest = &t[prefix.len()..];
        Some(rest.trim())
    } else if t.starts_with(&lp) {
        Some(t[lp.len()..].trim())
    } else {
        None
    }
}

fn parse_heading(line: &str) -> Option<(u32, &str, Vec<&str>)> {
    let t = line.trim_start();
    let depth = t.chars().take_while(|&c| c == '*').count() as u32;
    if depth == 0 {
        return None;
    }
    let rest = &t[depth as usize..];
    if !rest.starts_with(' ') {
        return None;
    }
    let text = rest.trim_start();
    let (heading, tags) = split_tags(text);
    Some((depth, heading, tags))
}

fn split_tags(text: &str) -> (&str, Vec<&str>) {
    if text.ends_with(':') {
        if let Some(pos) = text.rfind("  :").or_else(|| text.rfind("\t:")) {
            let tags_str = &text[pos + 2..]; // includes leading ":"
            let inner = &tags_str[1..tags_str.len() - 1];
            let tags: Vec<&str> = inner.split(':').collect();
            let heading = text[..pos].trim_end();
            return (heading, tags);
        }
        // Also handle direct ":noexport:" without double space
        if let Some(pos) = text.rfind(" :") {
            let tags_str = &text[pos + 1..];
            if tags_str.ends_with(':') && tags_str.starts_with(':') {
                let inner = &tags_str[1..tags_str.len() - 1];
                let tags: Vec<&str> = inner.split(':').collect();
                let heading = text[..pos].trim_end();
                return (heading, tags);
            }
        }
    }
    (text, vec![])
}

fn is_unordered_item(line: &str) -> bool {
    let t = line.trim_start();
    (t.starts_with("- ") || t.starts_with("+ ")) && t.len() > 2
}

fn unordered_content(line: &str) -> &str {
    let t = line.trim_start();
    &t[2..]
}

fn is_ordered_item(line: &str) -> bool {
    let t = line.trim_start();
    let b = t.as_bytes();
    let mut i = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && b.get(i) == Some(&b'.') && b.get(i + 1) == Some(&b' ')
}

fn ordered_parts(line: &str) -> (&str, &str) {
    let t = line.trim_start();
    if let Some(dot) = t.find(". ") {
        (&t[..dot], &t[dot + 2..])
    } else {
        ("1", t)
    }
}

fn strip_prefix_spaces(s: &str, n: usize) -> &str {
    let mut count = 0;
    let b = s.as_bytes();
    while count < n && count < b.len() && (b[count] == b' ' || b[count] == b'\t') {
        count += 1;
    }
    &s[count..]
}

/// Extract markup delimited by `delim`. Returns (inner_string, bytes_consumed).
/// Requires non-whitespace after opening delim and non-whitespace before closing.
/// The opening delimiter must be preceded by whitespace, start of text, or punctuation.
fn markup_at(chars: &[char], start: usize, delim: char) -> Option<(String, usize)> {
    if chars[start] != delim {
        return None;
    }
    if chars.len() <= start + 1 {
        return None;
    }
    // Opening delimiter must be preceded by whitespace/start/punctuation
    // but NOT by ':' (would be a URL like http://) or '/' (path)
    if start > 0 {
        let prev = chars[start - 1];
        if prev.is_alphanumeric() || prev == ':' || prev == '/' || prev == '~' {
            return None;
        }
    }
    if chars[start + 1].is_whitespace() {
        return None;
    }

    let mut i = start + 1;
    while i < chars.len() {
        if chars[i] == delim && !chars[i - 1].is_whitespace() && i > start + 1 {
            // Closing delimiter must be followed by whitespace/end/punctuation
            let after_ok = if i + 1 < chars.len() {
                let next = chars[i + 1];
                !next.is_alphanumeric() || matches!(next, ',' | '.' | ';' | ':' | '!' | '?')
            } else {
                true
            };
            if after_ok {
                let inner: String = chars[start + 1..i].iter().collect();
                // For slash-delimited italic, inner should not contain slashes (would be a path)
                if delim == '/' && inner.contains('/') {
                    i += 1;
                    continue;
                }
                let consumed: usize = i - start + 1; // char count (not bytes)
                return Some((inner, consumed));
            }
        }
        i += 1;
    }
    None
}

/// Percent-encode a path: encode non-ASCII and reserved chars except / - _ .
fn pct_encode(path: &str) -> String {
    let mut out = String::new();
    for c in path.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '+') {
            out.push(c);
        } else {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            for b in s.bytes() {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

/// Escape parentheses and ampersands in URLs for MDX compatibility
fn escape_url_parens(url: &str) -> String {
    url.replace('(', "\\(")
        .replace(')', "\\)")
        .replace('&', "\\&")
}

/// Convert org HTML snippet to JSX inline
fn html_to_jsx(html: &str) -> String {
    let trimmed = html.trim();
    // <!--more--> variants: map spaces inside to spaces in output
    if trimmed == "<!--more-->" {
        return "{/*more*/}".to_string();
    }
    if trimmed == "<!-- more -->" {
        return "{/* more */}".to_string();
    }
    let mut s = html.to_string();
    // class= -> className=
    s = s.replace(" class=\"", " className=\"");
    s = s.replace(" class='", " className='");
    // Remove spaces between consecutive closing tags (</div> </div> -> </div></div>)
    while s.contains("> </") {
        s = s.replace("> </", "></");
    }
    // self-close void elements
    for tag in &["br", "hr", "img", "input"] {
        s = s.replace(&format!("<{}>", tag), &format!("<{} />", tag));
        s = s.replace(&format!("<{}/>", tag), &format!("<{} />", tag));
    }
    s
}

/// Quote a string for YAML frontmatter
fn yaml_str(s: &str) -> String {
    let needs_quote = s.contains('"')
        || s.contains(':')
        || s.starts_with('\'')
        || s.starts_with('{')
        || s.starts_with('[');
    if needs_quote {
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

/// Parse org timestamp <YYYY-MM-DD Day HH:MM> and convert to UTC ISO8601
fn org_date_to_iso(s: &str) -> Option<String> {
    let s = s.trim().trim_start_matches('<').trim_end_matches('>');
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let date_str = parts[0];
    let time_str = parts
        .iter()
        .find(|p| p.contains(':'))
        .copied()
        .unwrap_or("00:00");

    let dp: Vec<&str> = date_str.split('-').collect();
    if dp.len() != 3 {
        return None;
    }
    let y: i32 = dp[0].parse().ok()?;
    let mo: u32 = dp[1].parse().ok()?;
    let d: u32 = dp[2].parse().ok()?;

    let tp: Vec<&str> = time_str.split(':').collect();
    let h: i32 = tp.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let m: i32 = tp.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    // CST = UTC+8, so UTC = CST - 8 hours
    let mut utc_h = h - 8;
    let mut utc_d = d as i32;
    let mut utc_mo = mo;
    let mut utc_y = y;

    if utc_h < 0 {
        utc_h += 24;
        utc_d -= 1;
        if utc_d < 1 {
            if utc_mo == 1 {
                utc_mo = 12;
                utc_y -= 1;
            } else {
                utc_mo -= 1;
            }
            utc_d = days_in_month(utc_y, utc_mo) as i32;
        }
    }

    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:00.000Z",
        utc_y, utc_mo, utc_d, utc_h, m
    ))
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_conversion() {
        assert_eq!(
            org_date_to_iso("<2019-09-30 Mon 11:20>"),
            Some("2019-09-30T03:20:00.000Z".to_string())
        );
        assert_eq!(
            org_date_to_iso("<2016-01-03 Sun 00:22>"),
            Some("2016-01-02T16:22:00.000Z".to_string())
        );
        assert_eq!(
            org_date_to_iso("<2013-08-04 Sun 23:28>"),
            Some("2013-08-04T15:28:00.000Z".to_string())
        );
    }

    #[test]
    fn test_pct_encode() {
        assert_eq!(pct_encode("foo/bar.png"), "foo/bar.png");
        assert_eq!(
            pct_encode("使用-opencode/img.png"),
            "%E4%BD%BF%E7%94%A8-opencode/img.png"
        );
    }
}
