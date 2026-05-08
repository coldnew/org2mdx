use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

/// Resolves `#+INCLUDE:` directives in the input text by reading referenced files
/// and inserting their content in place of the directive.
///
/// Supports:
/// - `#+INCLUDE: "file.org"` — include entire file
/// - `#+INCLUDE: file.org` — same without quotes (when filename has no spaces)
/// - `#+INCLUDE: "file.org" :lines "5-10"` — include lines 5 through 10
/// - `#+INCLUDE: "file.org" :lines "5-"` — include from line 5 to end
/// - `#+INCLUDE: "file.org" :lines "-10"` — include first 10 lines
///
/// Recursive includes are resolved up to `max_depth` (default 5).
/// `base_dir` is used to resolve relative file paths, and should
/// typically be the directory of the input Org file.
pub fn resolve_includes(input: &str, base_dir: &Path) -> Result<String> {
    resolve_includes_depth(input, base_dir, 0, 5)
}

fn resolve_includes_depth(
    input: &str,
    base_dir: &Path,
    depth: usize,
    max_depth: usize,
) -> Result<String> {
    if depth >= max_depth {
        return Err(Error::InvalidInput(format!(
            "#+INCLUDE: exceeded maximum recursion depth of {}",
            max_depth
        )));
    }

    let mut result = String::with_capacity(input.len());
    for line in input.lines() {
        let trimmed = line.trim();
        if let Some(include_dir) = parse_include_directive(trimmed) {
            let resolved_path = resolve_path(&include_dir.file, base_dir)?;
            let file_content = std::fs::read_to_string(&resolved_path).map_err(|e| {
                Error::InvalidInput(format!(
                    "#+INCLUDE: cannot read {}: {}",
                    resolved_path.display(),
                    e
                ))
            })?;

            let included_content = resolve_includes_depth(
                &file_content,
                &resolved_path.parent().unwrap_or(base_dir),
                depth + 1,
                max_depth,
            )?;

            let filtered = apply_line_filter(&included_content, include_dir.line_range);
            result.push_str(&filtered);
            // Ensure trailing newline so included content doesn't merge with the next line
            if !filtered.ends_with('\n') {
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    Ok(result)
}

struct IncludeDirective {
    file: String,
    line_range: Option<(Option<usize>, Option<usize>)>, // (start, end) 1-based, None = no bound
}

/// Parse an `#+INCLUDE:` directive line.
/// Returns None if the line is not an include directive.
fn parse_include_directive(line: &str) -> Option<IncludeDirective> {
    let rest = line.strip_prefix("#+INCLUDE:")?;
    let rest = rest.trim();

    if rest.is_empty() {
        return None;
    }

    // Parse filename: either quoted "file.org" or unquoted until whitespace
    let (file, remainder) = if rest.starts_with('"') {
        let end_quote = rest[1..].find('"')?;
        let file = &rest[1..1 + end_quote];
        let remainder = rest[1 + end_quote + 1..].trim();
        (file.to_string(), remainder)
    } else {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let file = &rest[..end];
        let remainder = rest[end..].trim();
        (file.to_string(), remainder)
    };

    // Parse optional parameters
    let mut line_range = None;
    let mut remaining = remainder;
    while !remaining.is_empty() {
        if let Some(val) = strip_param(remaining, ":lines") {
            line_range = Some(parse_line_range(val)?);
            // Find next param (starts with ':')
            remaining = find_next_param(remaining, ":lines", val);
        } else {
            break;
        }
    }

    Some(IncludeDirective { file, line_range })
}

/// Strip a parameter value from the beginning of `s`.
/// Returns the value if `s` starts with `param_name ` and the value.
fn strip_param<'a>(s: &'a str, param_name: &str) -> Option<&'a str> {
    let after_name = s.strip_prefix(param_name)?;
    let after_name = after_name.trim_start();
    if after_name.starts_with('"') {
        let end_quote = after_name[1..].find('"')?;
        Some(&after_name[1..1 + end_quote])
    } else {
        let end = after_name
            .find(|c: char| c.is_whitespace())
            .unwrap_or(after_name.len());
        if end == 0 {
            None
        } else {
            Some(&after_name[..end])
        }
    }
}

/// Find the start of the next parameter after consuming `param_name` and `param_val`.
fn find_next_param<'a>(s: &'a str, param_name: &str, param_val: &str) -> &'a str {
    let after_name = s.strip_prefix(param_name).unwrap_or(s);
    let after_name = after_name.trim_start();
    if after_name.starts_with('"') {
        // Skip the quoted value
        if let Some(end_quote) = after_name[1..].find('"') {
            let after_val = &after_name[1 + end_quote + 1..];
            return after_val.trim();
        }
    } else {
        // Skip the unquoted value
        if let Some(val_pos) = after_name.find(param_val) {
            let after_val = &after_name[val_pos + param_val.len()..];
            return after_val.trim();
        }
    }
    ""
}

/// Parse a line range string like "5-10", "5-", "-10", or "5".
/// Returns None if the format is invalid.
fn parse_line_range(s: &str) -> Option<(Option<usize>, Option<usize>)> {
    if s.is_empty() {
        return None;
    }
    if let Some(dash_pos) = s.find('-') {
        let start_str = &s[..dash_pos];
        let end_str = &s[dash_pos + 1..];
        let start = if start_str.is_empty() {
            None
        } else {
            Some(start_str.parse::<usize>().ok()?)
        };
        let end = if end_str.is_empty() {
            None
        } else {
            Some(end_str.parse::<usize>().ok()?)
        };
        if start.is_none() && end.is_none() {
            return None;
        }
        Some((start, end))
    } else {
        // Single number: include that specific line
        let line = s.parse::<usize>().ok()?;
        Some((Some(line), Some(line)))
    }
}

/// Apply line filtering to content based on the line range.
fn apply_line_filter(content: &str, range: Option<(Option<usize>, Option<usize>)>) -> String {
    let Some((start, end)) = range else {
        return content.to_string();
    };

    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();
    if total == 0 {
        return String::new();
    }

    let start_idx = match start {
        Some(s) if s >= 1 => (s - 1).min(total),
        None => 0,
        _ => 0,
    };
    let end_idx = match end {
        Some(e) if e >= 1 => e.min(total),
        None => total,
        _ => total,
    };

    if start_idx >= end_idx {
        return String::new();
    }

    let mut result = String::new();
    for line in &lines[start_idx..end_idx] {
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Resolve a file path relative to `base_dir`.
/// Handles both absolute paths and relative paths.
fn resolve_path(file: &str, base_dir: &Path) -> Result<PathBuf> {
    let path = Path::new(file);
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(base_dir.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_include_with_quotes() {
        let dir = parse_include_directive("#+INCLUDE: \"file.org\"").unwrap();
        assert_eq!(dir.file, "file.org");
        assert!(dir.line_range.is_none());
    }

    #[test]
    fn test_parse_include_without_quotes() {
        let dir = parse_include_directive("#+INCLUDE: file.org").unwrap();
        assert_eq!(dir.file, "file.org");
        assert!(dir.line_range.is_none());
    }

    #[test]
    fn test_parse_include_with_lines() {
        let dir = parse_include_directive("#+INCLUDE: \"file.org\" :lines \"5-10\"").unwrap();
        assert_eq!(dir.file, "file.org");
        assert_eq!(dir.line_range, Some((Some(5), Some(10))));
    }

    #[test]
    fn test_parse_include_lines_from_start() {
        let dir = parse_include_directive("#+INCLUDE: \"file.org\" :lines \"-5\"").unwrap();
        assert_eq!(dir.file, "file.org");
        assert_eq!(dir.line_range, Some((None, Some(5))));
    }

    #[test]
    fn test_parse_include_lines_to_end() {
        let dir = parse_include_directive("#+INCLUDE: \"file.org\" :lines \"5-\"").unwrap();
        assert_eq!(dir.file, "file.org");
        assert_eq!(dir.line_range, Some((Some(5), None)));
    }

    #[test]
    fn test_parse_include_single_line() {
        let dir = parse_include_directive("#+INCLUDE: \"file.org\" :lines \"5\"").unwrap();
        assert_eq!(dir.file, "file.org");
        assert_eq!(dir.line_range, Some((Some(5), Some(5))));
    }

    #[test]
    fn test_apply_line_filter_full() {
        let content = "line1\nline2\nline3\n";
        let result = apply_line_filter(content, None);
        assert_eq!(result, "line1\nline2\nline3\n");
    }

    #[test]
    fn test_apply_line_filter_range() {
        let content = "line1\nline2\nline3\nline4\n";
        let result = apply_line_filter(content, Some((Some(2), Some(3))));
        assert_eq!(result, "line2\nline3\n");
    }

    #[test]
    fn test_apply_line_filter_from_start() {
        let content = "line1\nline2\nline3\n";
        let result = apply_line_filter(content, Some((None, Some(2))));
        assert_eq!(result, "line1\nline2\n");
    }

    #[test]
    fn test_apply_line_filter_to_end() {
        let content = "line1\nline2\nline3\n";
        let result = apply_line_filter(content, Some((Some(2), None)));
        assert_eq!(result, "line2\nline3\n");
    }

    #[test]
    fn test_resolve_includes_basic() {
        let tmp = std::env::temp_dir().join("org2mdx_test_include");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let included_path = tmp.join("included.org");
        fs::write(&included_path, "included content\n").unwrap();

        let main_path = tmp.join("main.org");
        fs::write(
            &main_path,
            format!("#+INCLUDE: \"{}\"\n", included_path.display()),
        )
        .unwrap();

        let input = fs::read_to_string(&main_path).unwrap();
        let resolved = resolve_includes(&input, &tmp).unwrap();
        assert_eq!(resolved.trim(), "included content");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_resolve_includes_with_lines() {
        let tmp = std::env::temp_dir().join("org2mdx_test_include_lines");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let included_path = tmp.join("included.org");
        fs::write(&included_path, "line1\nline2\nline3\nline4\n").unwrap();

        let main_path = tmp.join("main.org");
        fs::write(
            &main_path,
            format!(
                "#+INCLUDE: \"{}\" :lines \"2-3\"\n",
                included_path.display()
            ),
        )
        .unwrap();

        let input = fs::read_to_string(&main_path).unwrap();
        let resolved = resolve_includes(&input, &tmp).unwrap();
        assert_eq!(resolved.trim(), "line2\nline3");

        let _ = fs::remove_dir_all(&tmp);
    }
}
