pub fn parse_heading(line: &str) -> Option<(u32, &str, Vec<&str>)> {
    let t = line.trim();
    let depth = t.chars().take_while(|c| *c == '*').count() as u32;
    if depth == 0 {
        return None;
    }
    let rest = &t[depth as usize..];
    if !rest.starts_with(' ') {
        return None;
    }
    let text = rest[1..].trim();
    let (heading, tags) = split_tags(text);
    Some((depth, heading, tags))
}

pub fn split_tags(text: &str) -> (&str, Vec<&str>) {
    if text.ends_with(':') {
        if let Some(pos) = text.rfind("  :").or_else(|| text.rfind("\t:")) {
            let tags_str = &text[pos + 2..];
            let inner = &tags_str[1..tags_str.len() - 1];
            let tags: Vec<&str> = inner.split(':').collect();
            let heading = text[..pos].trim_end();
            return (heading, tags);
        }
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

pub fn should_skip_section(tags: &[&str]) -> bool {
    tags.contains(&"noexport") || tags.contains(&"skip")
}
