use crate::util::strip_prefix_spaces;

pub enum BlockType {
    Src(String),
    Example,
    Quote,
    Center,
    Export,
    Unknown(String),
}

impl BlockType {
    pub fn end_keyword(&self) -> String {
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

pub fn parse_block_begin(line: &str) -> BlockType {
    let lower = line.trim().to_lowercase();
    if let Some(rest) = lower.strip_prefix("#+begin_src") {
        let lang = rest.trim().to_string();
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

pub fn render_block(
    bt: &BlockType,
    lines: &[String],
    inline_fn: impl Fn(&str) -> String,
) -> String {
    let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
    let lines = &lines[start..];
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
                    s.push_str(&format!("> {}", inline_fn(t)));
                }
            }
            s
        }
        BlockType::Center => {
            let parts: Vec<String> = lines
                .iter()
                .filter(|l| !l.trim().is_empty())
                .map(|l| inline_fn(l.trim()))
                .collect();
            parts.join(" ")
        }
        BlockType::Export => String::new(),
        BlockType::Unknown(_) => {
            let parts: Vec<String> = lines
                .iter()
                .filter(|l| !l.trim().is_empty())
                .map(|l| inline_fn(l.trim()))
                .collect();
            parts.join(" ")
        }
    }
}
