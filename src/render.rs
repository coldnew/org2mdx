pub fn push_block(out: &mut String, content: &str) {
    push_block_n(out, content, 1);
}

pub fn push_block_n(out: &mut String, content: &str, min_blanks: usize) {
    if content.is_empty() {
        return;
    }
    let needed_newlines = min_blanks + 1;
    if out.is_empty() {
        out.push('\n');
    } else {
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

pub fn push_block_exact(out: &mut String, content: &str, blanks: usize) {
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
