/// JSX block detection helpers for MDX parsing.
/// These detect JSX component tags, import/export statements, and determine
/// block boundaries in source text — no markdown or mdast dependencies.

/// Returns true if the line starts with `<` followed by an uppercase letter — indicating a JSX
/// component tag at the start of a line (block-level JSX, not inline within a paragraph).
pub fn is_jsx_anchor_line(line: &str) -> bool {
    if let Some(rest) = line.strip_prefix('<') {
        rest.chars().next().is_some_and(|c| c.is_uppercase())
    } else {
        false
    }
}

/// Returns true if the line begins a JSX block: starts with a JSX component tag,
/// `import `, or `export ` after trimming.
pub fn is_jsx_line(line: &str) -> bool {
    let trimmed = line.trim();
    is_jsx_anchor_line(trimmed) || trimmed.starts_with("import ") || trimmed.starts_with("export ")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_jsx_anchor_line() {
        assert!(is_jsx_anchor_line("<Foo>"));
        assert!(is_jsx_anchor_line("<Button color=\"red\">"));
        assert!(is_jsx_anchor_line("<Callout>Hi</Callout>"));
        assert!(!is_jsx_anchor_line("<div>"));
        assert!(!is_jsx_anchor_line("<>"));
        assert!(!is_jsx_anchor_line("3 < 5"));
        assert!(!is_jsx_anchor_line("not jsx"));
        assert!(!is_jsx_anchor_line(""));
    }

    #[test]
    fn test_is_jsx_line() {
        assert!(is_jsx_line("<Foo>"));
        assert!(is_jsx_line("import { X } from 'y'"));
        assert!(is_jsx_line("export default Foo"));
        assert!(!is_jsx_line("<div>"));
        assert!(!is_jsx_line("some text"));
    }

    #[test]
    fn test_find_jsx_block_start_with_imports() {
        let lines: Vec<&str> = vec!["import { X } from 'y'", "<Foo>bar</Foo>"];
        // anchor at line 1 (<Foo>)
        let start = find_jsx_block_start(&lines, 1);
        assert_eq!(start, 0); // should include the import line
    }

    #[test]
    fn test_find_jsx_block_start_no_import() {
        let lines: Vec<&str> = vec!["some text", "<Foo>bar</Foo>"];
        let start = find_jsx_block_start(&lines, 1);
        assert_eq!(start, 1); // should stay at the anchor
    }

    #[test]
    fn test_find_jsx_block_start_with_blank_line() {
        let lines: Vec<&str> = vec!["import { X } from 'y'", "", "<Foo>bar</Foo>"];
        // anchor at line 2
        let start = find_jsx_block_start(&lines, 2);
        assert_eq!(start, 0); // skip blank line to include import
    }

    #[test]
    fn test_find_jsx_block_end_single_line() {
        let lines: Vec<&str> = vec!["<Foo>bar</Foo>", "some text"];
        let end = find_jsx_block_end(&lines, 0);
        assert_eq!(end, 0); // only the anchor line
    }

    #[test]
    fn test_find_jsx_block_end_multiple_lines() {
        // Note: closing tags (</Foo>) are NOT detected as JSX anchor lines
        // because is_jsx_anchor_line requires <Uppercase>, not </
        let lines: Vec<&str> = vec!["<Foo>", "  <Bar />", "</Foo>", "some text"];
        let end = find_jsx_block_end(&lines, 0);
        assert_eq!(end, 1); // only <Foo> and <Bar /> (both <Uppercase>)
                            // </Foo> starts with '</' so it's not a JSX anchor
    }

    #[test]
    fn test_find_jsx_block_end_with_blank_line() {
        let lines: Vec<&str> = vec!["<Foo>bar</Foo>", "", "<Baz />", "some text"];
        let end = find_jsx_block_end(&lines, 0);
        assert_eq!(end, 2); // includes blank line and <Baz />
    }
}
