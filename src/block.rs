pub enum BlockType {
    Src(String),
    Example,
    Quote,
    Center,
    Export(String),
    Unknown(String),
}

impl BlockType {
    pub fn end_keyword(&self) -> String {
        match self {
            BlockType::Src(_) => "src".to_string(),
            BlockType::Example => "example".to_string(),
            BlockType::Quote => "quote".to_string(),
            BlockType::Center => "center".to_string(),
            BlockType::Export(_) => "export".to_string(),
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
        let export_type = line
            .trim()
            .strip_prefix("#+begin_export")
            .unwrap_or("")
            .trim()
            .to_string();
        BlockType::Export(export_type)
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
