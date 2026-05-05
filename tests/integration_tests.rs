use org2mdx::ast::Node;
use serde_json::Value;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct AstFixture {
    name: String,
    org_path: PathBuf,
    ast_path: PathBuf,
    mdx_path: PathBuf,
}

#[test]
fn test_org_to_ast_fixtures() {
    let fixtures = load_ast_fixtures();
    let mut failures = Vec::new();

    for fixture in fixtures {
        let org = read_file(&fixture.org_path);
        let expected_ast = read_json(&fixture.ast_path);

        let actual_ast = match org2mdx::org_to_ast::parse(&org) {
            Ok(node) => normalize_ast(json_of_node(&node)),
            Err(e) => {
                failures.push(format!("{} org->ast parse failed: {}", fixture.name, e));
                continue;
            }
        };

        if actual_ast != expected_ast {
            failures.push(format!(
                "{} org->ast mismatch\nexpected: {}\nactual:   {}",
                fixture.name,
                pretty_json(&expected_ast),
                pretty_json(&actual_ast)
            ));
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n\n"));
    }
}

#[test]
fn test_ast_to_mdx_fixtures() {
    let fixtures = load_ast_fixtures();
    let mut failures = Vec::new();

    for fixture in fixtures {
        let ast = read_json(&fixture.ast_path);
        let expected_mdx = read_file(&fixture.mdx_path);

        let node: Node = match serde_json::from_value(ast) {
            Ok(node) => node,
            Err(e) => {
                failures.push(format!("{} invalid AST fixture: {}", fixture.name, e));
                continue;
            }
        };

        let actual_mdx = org2mdx::renderer::mdx_renderer::render_mdx(&node);
        if actual_mdx != expected_mdx {
            failures.push(format!(
                "{} ast->mdx mismatch\nexpected:\n{}\nactual:\n{}",
                fixture.name, expected_mdx, actual_mdx
            ));
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n\n"));
    }
}

#[test]
fn test_mdx_to_ast_fixtures() {
    let fixtures = load_ast_fixtures();
    let mut failures = Vec::new();

    for fixture in fixtures {
        let mdx = read_file(&fixture.mdx_path);
        let expected_ast = read_json(&fixture.ast_path);

        let actual_ast = match org2mdx::mdx_to_ast::parse(&mdx) {
            Ok(node) => normalize_ast(json_of_node(&node)),
            Err(e) => {
                failures.push(format!("{} mdx->ast parse failed: {}", fixture.name, e));
                continue;
            }
        };

        if actual_ast != expected_ast {
            failures.push(format!(
                "{} mdx->ast mismatch\nexpected: {}\nactual:   {}",
                fixture.name,
                pretty_json(&expected_ast),
                pretty_json(&actual_ast)
            ));
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n\n"));
    }
}

#[test]
fn test_standard_org_to_mdx_fixtures() {
    let org_dir = Path::new("tests/org");
    let mdx_dir = Path::new("tests/mdx");
    let entries =
        fs::read_dir(org_dir).unwrap_or_else(|e| panic!("cannot read {}: {}", org_dir.display(), e));

    let mut failures = Vec::new();

    for entry in entries {
        let org_path = entry.expect("invalid directory entry").path();
        if org_path.extension().and_then(|s| s.to_str()) != Some("org") {
            continue;
        }

        let stem = org_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let mdx_path = mdx_dir.join(format!("{}.mdx", stem));
        if !mdx_path.exists() {
            failures.push(format!("missing expected fixture {}", mdx_path.display()));
            continue;
        }

        let org = read_file(&org_path);
        let expected_mdx = read_file(&mdx_path);
        let actual_mdx = match org2mdx::org_to_mdx::convert(&org) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("{} org->mdx conversion failed: {}", stem, e));
                continue;
            }
        };

        if let Err(reason) = compare_mdx_with_frontmatter(&expected_mdx, &actual_mdx) {
            failures.push(format!("{} org->mdx mismatch: {}", stem, reason));
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n\n"));
    }
}

#[test]
fn test_standard_mdx_to_org_fixtures() {
    let org_dir = Path::new("tests/org");
    let mdx_dir = Path::new("tests/mdx");
    let entries =
        fs::read_dir(mdx_dir).unwrap_or_else(|e| panic!("cannot read {}: {}", mdx_dir.display(), e));

    let mut failures = Vec::new();

    for entry in entries {
        let mdx_path = entry.expect("invalid directory entry").path();
        if mdx_path.extension().and_then(|s| s.to_str()) != Some("mdx") {
            continue;
        }

        let stem = mdx_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let org_path = org_dir.join(format!("{}.org", stem));
        if !org_path.exists() {
            failures.push(format!("missing expected fixture {}", org_path.display()));
            continue;
        }

        let mdx = read_file(&mdx_path);
        let expected_org = read_file(&org_path);

        let actual_org = match org2mdx::mdx_to_org::convert(&mdx) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("{} mdx->org conversion failed: {}", stem, e));
                continue;
            }
        };

        if let Err(reason) = compare_org_with_link_tolerance(&expected_org, &actual_org) {
            failures.push(format!("{} mdx->org mismatch: {}", stem, reason));
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n\n"));
    }
}

fn load_ast_fixtures() -> Vec<AstFixture> {
    let dir = Path::new("tests/ast");
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {}", dir.display(), e));

    let mut fixtures = Vec::new();
    for entry in entries {
        let path = entry.expect("invalid directory entry").path();
        if path.extension().and_then(|s| s.to_str()) != Some("org") {
            continue;
        }

        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let ast_path = dir.join(format!("{}.ast", stem));
        let mdx_path = dir.join(format!("{}.mdx", stem));

        if !ast_path.exists() {
            panic!("missing fixture file: {}", ast_path.display());
        }
        if !mdx_path.exists() {
            panic!("missing fixture file: {}", mdx_path.display());
        }

        fixtures.push(AstFixture {
            name: stem,
            org_path: path,
            ast_path,
            mdx_path,
        });
    }

    fixtures.sort_by(|a, b| a.name.cmp(&b.name));
    assert!(!fixtures.is_empty(), "no fixtures found in tests/ast");
    fixtures
}

fn normalize_ast(value: Value) -> Value {
    match value {
        Value::Object(mut map) => {
            if let Some(children) = map.remove("children") {
                let normalized_children = normalize_children(children);
                if !normalized_children.is_empty() {
                    map.insert("children".to_string(), Value::Array(normalized_children));
                }
            }

            if let Some(data) = map.remove("data") {
                let normalized_data = normalize_data(data);
                if !normalized_data.is_null() {
                    map.insert("data".to_string(), normalized_data);
                }
            }

            Value::Object(map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(normalize_ast).collect()),
        other => other,
    }
}

fn normalize_children(children: Value) -> Vec<Value> {
    let arr = match children {
        Value::Array(arr) => arr,
        _ => return Vec::new(),
    };

    let mut normalized = Vec::new();
    for child in arr.into_iter().map(normalize_ast) {
        if let Value::Object(obj) = &child {
            if obj.get("type").and_then(Value::as_str) == Some("blankLine") {
                continue;
            }
        }

        normalized.push(child);
    }

    merge_adjacent_text_nodes(normalized)
}

fn normalize_data(data: Value) -> Value {
    let mut obj = match data {
        Value::Object(obj) => obj,
        _ => return Value::Null,
    };

    obj.remove("tags");
    if obj.is_empty() {
        return Value::Null;
    }

    Value::Object(obj)
}

fn merge_adjacent_text_nodes(children: Vec<Value>) -> Vec<Value> {
    let mut merged = Vec::new();

    for child in children {
        let text = child
            .get("type")
            .and_then(Value::as_str)
            .and_then(|t| if t == "text" { child.get("value").and_then(Value::as_str) } else { None })
            .map(|s| s.to_string());

        if let Some(text_value) = text {
            if let Some(Value::Object(last)) = merged.last_mut() {
                let is_last_text = last.get("type").and_then(Value::as_str) == Some("text");
                if is_last_text {
                    if let Some(Value::String(last_value)) = last.get_mut("value") {
                        last_value.push_str(&text_value);
                        continue;
                    }
                }
            }
        }

        merged.push(child);
    }

    merged
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {}", path.display(), e))
}

fn read_json(path: &Path) -> Value {
    let raw = read_file(path);
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("cannot parse json {}: {}", path.display(), e))
}

fn json_of_node(node: &Node) -> Value {
    serde_json::to_value(node).expect("failed to convert node to json")
}

fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).expect("failed to format json")
}

fn compare_mdx_with_frontmatter(expected: &str, actual: &str) -> Result<(), String> {
    let (expected_fm, expected_body) = split_yaml_frontmatter(expected)
        .ok_or_else(|| "expected fixture has invalid frontmatter".to_string())?;
    let (actual_fm, actual_body) = split_yaml_frontmatter(actual)
        .ok_or_else(|| "actual output has invalid frontmatter".to_string())?;

    if expected_fm != actual_fm {
        return Err(format!(
            "frontmatter differs\nexpected: {:#?}\nactual:   {:#?}",
            expected_fm, actual_fm
        ));
    }

    if expected_body != actual_body {
        return Err(format!(
            "body differs\nexpected:\n{}\nactual:\n{}",
            expected_body, actual_body
        ));
    }

    Ok(())
}

fn compare_org_with_link_tolerance(expected: &str, actual: &str) -> Result<(), String> {
    if expected == actual {
        return Ok(());
    }

    let expected_links: Vec<&str> = expected
        .lines()
        .filter(|line| line.trim_start().starts_with("#+LINK:"))
        .collect();
    let actual_links: Vec<&str> = actual
        .lines()
        .filter(|line| line.trim_start().starts_with("#+LINK:"))
        .collect();

    let mut expected_links_sorted = expected_links.clone();
    expected_links_sorted.sort_unstable();
    let mut actual_links_sorted = actual_links.clone();
    actual_links_sorted.sort_unstable();

    let expected_without_links = expected
        .lines()
        .filter(|line| !line.trim_start().starts_with("#+LINK:"))
        .collect::<Vec<_>>()
        .join("\n");
    let actual_without_links = actual
        .lines()
        .filter(|line| !line.trim_start().starts_with("#+LINK:"))
        .collect::<Vec<_>>()
        .join("\n");

    if expected_without_links == actual_without_links && expected_links_sorted == actual_links_sorted {
        return Ok(());
    }

    Err(format!("expected:\n{}\nactual:\n{}", expected, actual))
}

fn split_yaml_frontmatter(input: &str) -> Option<(YamlValue, &str)> {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---\n") {
        return None;
    }

    let after_start = &trimmed[4..];
    let end = after_start.find("\n---\n")?;
    let yaml_part = &after_start[..end];
    let rest = &after_start[end + 5..];
    let parsed = serde_yaml::from_str(yaml_part).ok()?;
    Some((parsed, rest))
}
