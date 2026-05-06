/// JSX block detection helpers for MDX parsing.
/// These detect JSX component tags, import/export statements, and determine
/// block boundaries in source text — no markdown or mdast dependencies.

/// Returns true if the line starts with `<` followed by an uppercase letter — indicating a JSX
/// component tag at the start of a line (block-level JSX, not inline within a paragraph).
pub fn is_jsx_anchor_line(line: &str) -> bool {
    if let Some(rest) = line.strip_prefix('<') {
        rest.chars().next().map_or(false, |c| c.is_uppercase())
    } else {
        false
    }
}

/// Returns true if the line begins a JSX block: starts with a JSX component tag,
/// `import `, or `export ` after trimming.
pub fn is_jsx_line(line: &str) -> bool {
    let trimmed = line.trim();
    is_jsx_anchor_line(trimmed)
        || trimmed.starts_with("import ")
        || trimmed.starts_with("export ")
}
/// Scan backward from `anchor` to find the start of a JSX block, including preceding
/// `import`/`export` lines with optional blank lines between them. Stops at the first
/// non-import/export line that isn't separated by only blank lines from the block.
pub fn find_jsx_block_start(lines: &[&str], anchor: usize) -> usize {
    let mut start = anchor;
    while start > 0 {
        let prev = lines[start - 1].trim();
        if prev.starts_with("import ") || prev.starts_with("export ") {
            start -= 1;
        } else if prev.is_empty() {
            // skip consecutive blank lines to the next import/export, or stop
            let mut candidate = start - 1;
            while candidate > 0 && lines[candidate].trim().is_empty() {
                candidate -= 1;
            }
            if lines[candidate].trim().starts_with("import ")
                || lines[candidate].trim().starts_with("export ")
            {
                start = candidate;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    start
}

/// Scan forward from `anchor` to find the end of a JSX block, including subsequent
/// JSX lines with optional blank lines between them.
pub fn find_jsx_block_end(lines: &[&str], anchor: usize) -> usize {
    let mut end = anchor;
    while end + 1 < lines.len() {
        let next = lines[end + 1].trim();
        if is_jsx_line(next) {
            end += 1;
        } else if next.is_empty() && end + 2 < lines.len() && is_jsx_line(lines[end + 2].trim()) {
            end += 2;
        } else {
            break;
        }
    }
    end
}
