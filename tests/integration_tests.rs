use org2mdx::ast::Node;
use serde_json::Value;
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
