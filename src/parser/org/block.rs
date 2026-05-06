pub struct SrcOptions {
    pub lang: String,
    pub exports: Option<String>,
}

pub struct ExportOptions {
    pub export_type: String,
    pub exports: Option<String>,
}

pub enum BlockType {
    Src(SrcOptions),
    Example,
    Quote,
    Center,
    Export(ExportOptions),
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
        let parts: Vec<&str> = rest.trim().split_whitespace().collect();
        let lang = match parts.first().copied().unwrap_or("") {
            "c++" | "cpp" => "c".to_string(),
            other => other.to_string(),
        };
        let mut exports = None;
        let mut i = 1;
        while i < parts.len() {
            let token = parts[i];
            if let Some(key) = token.strip_prefix(':') {
                if key == "exports" && i + 1 < parts.len() {
                    exports = Some(parts[i + 1].to_string());
                    i += 2;
                    continue;
                }
            }
            i += 1;
        }
        BlockType::Src(SrcOptions { lang, exports })
    } else if lower.starts_with("#+begin_example") {
        BlockType::Example
    } else if lower.starts_with("#+begin_quote") {
        BlockType::Quote
    } else if lower.starts_with("#+begin_center") {
        BlockType::Center
    } else if lower.starts_with("#+begin_export") {
        let rest = lower.strip_prefix("#+begin_export").unwrap_or("").trim();
        let parts: Vec<&str> = rest.split_whitespace().collect();
        let export_type = parts.first().copied().unwrap_or("").to_string();
        let mut exports = None;
        let mut i = 1;
        while i < parts.len() {
            let token = parts[i];
            if let Some(key) = token.strip_prefix(':') {
                if key == "exports" && i + 1 < parts.len() {
                    exports = Some(parts[i + 1].to_string());
                    i += 2;
                    continue;
                }
            }
            i += 1;
        }
        BlockType::Export(ExportOptions {
            export_type,
            exports,
        })
    } else {
        let name = lower.strip_prefix("#+begin_").unwrap_or(&lower).to_string();
        BlockType::Unknown(name)
    }
}
