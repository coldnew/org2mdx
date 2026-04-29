use crate::converter::OrgConverter;

pub trait ParagraphParser {
    fn collect_paragraph(&mut self) -> (String, bool);
}

impl ParagraphParser for OrgConverter {
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
                    if crate::heading::parse_heading(&l).is_some() {
                        break;
                    }
                    if crate::list::is_unordered_item(&l) {
                        break;
                    }
                    if crate::list::is_ordered_item(&l) {
                        break;
                    }
                    let trimmed = l.trim();
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
        let joined = parts.join(" ");
        let normalized = crate::util::collapse_spaces(&joined);
        (self.inline(&normalized), had_line_break)
    }
}
