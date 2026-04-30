use serde_yaml::Value;
use std::fs;
use std::path::Path;

// Dynamically discover all .org files in tests/org/ and test conversion against corresponding .mdx files.
// If a .mdx file is missing, the test fails.
#[test]
fn test_all_org_files() {
    let org_dir = Path::new("tests/org");
    let mdx_dir = Path::new("tests/mdx");

    let entries =
        fs::read_dir(org_dir).unwrap_or_else(|e| panic!("Cannot read tests/org directory: {}", e));

    let mut failures = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let org_path = entry.path();
        if org_path.extension().and_then(|s| s.to_str()) != Some("org") {
            continue;
        }
        let stem = org_path.file_stem().unwrap().to_str().unwrap();
        let mdx_path = mdx_dir.join(format!("{}.mdx", stem));

        if !mdx_path.exists() {
            failures.push(format!("Missing expected .mdx file for {}", stem));
            continue;
        }

        let org_content = fs::read_to_string(&org_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", org_path.display(), e));
        let expected = fs::read_to_string(&mdx_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", mdx_path.display(), e));

        let actual = match org2mdx::org_to_mdx::convert(&org_content) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("Conversion failed for {}: {}", stem, e));
                continue;
            }
        };

        // Split frontmatter and content
        fn split_frontmatter(s: &str) -> Option<(Value, &str)> {
            let s = s.trim_start();
            if !s.starts_with("---\n") {
                return None;
            }
            let after_start = &s[4..];
            if let Some(end) = after_start.find("\n---\n") {
                let yaml_str = &after_start[..end];
                let rest = &after_start[end + 5..];
                match serde_yaml::from_str(yaml_str) {
                    Ok(value) => Some((value, rest)),
                    Err(_) => None,
                }
            } else {
                None
            }
        }

        let (expected_fm, expected_rest) = split_frontmatter(&expected).unwrap_or_else(|| {
            panic!(
                "Expected file {} has invalid frontmatter",
                mdx_path.display()
            )
        });
        let (actual_fm, actual_rest) = split_frontmatter(&actual)
            .unwrap_or_else(|| panic!("Actual output for {} has invalid frontmatter", stem));

        // Compare frontmatter as YAML values (order-insensitive)
        if expected_fm != actual_fm {
            failures.push(format!(
                "Frontmatter mismatch for {}:\n  expected: {:#?}\n  actual:   {:#?}",
                stem, expected_fm, actual_fm
            ));
        }

        // Compare the rest of the content
        if expected_rest != actual_rest {
            let expected_lines: Vec<&str> = expected_rest.lines().collect();
            let actual_lines: Vec<&str> = actual_rest.lines().collect();
            let max = expected_lines.len().max(actual_lines.len());
            let mut diff = String::new();
            for i in 0..max {
                let e = expected_lines.get(i).copied().unwrap_or("<missing>");
                let a = actual_lines.get(i).copied().unwrap_or("<missing>");
                if a != e {
                    diff.push_str(&format!(
                        "\nLine {} mismatch:\n  expected: {:?}\n  actual:   {:?}",
                        i + 1,
                        e,
                        a
                    ));
                    if diff.len() > 500 {
                        diff.push_str("\n... (diff truncated)");
                        break;
                    }
                }
            }
            failures.push(format!("Content mismatch for {}:{}", stem, diff));
        }
    }

    if !failures.is_empty() {
        panic!("Test failures:\n{}", failures.join("\n"));
    }
}

// New test: verify that mdx -> org conversion works for all .mdx files
#[test]
fn test_mdx_to_org_conversion() {
    let mdx_dir = Path::new("tests/mdx");
    let org_dir = Path::new("tests/org"); // optional expected .org files

    let entries =
        fs::read_dir(mdx_dir).unwrap_or_else(|e| panic!("Cannot read tests/mdx directory: {}", e));

    let mut failures = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let mdx_path = entry.path();
        if mdx_path.extension().and_then(|s| s.to_str()) != Some("mdx") {
            continue;
        }
        let stem = mdx_path.file_stem().unwrap().to_str().unwrap();
        let expected_org_path = org_dir.join(format!("{}.org", stem));

        let mdx_content = fs::read_to_string(&mdx_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", mdx_path.display(), e));

        let actual_org = match org2mdx::mdx_to_org::convert(&mdx_content) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("MDX→Org conversion failed for {}: {}", stem, e));
                continue;
            }
        };

        // Basic sanity checks
        if actual_org.is_empty() {
            failures.push(format!("MDX→Org produced empty output for {}", stem));
        }

        // If an expected .org file exists, compare against it
        if expected_org_path.exists() {
            let expected_org = fs::read_to_string(&expected_org_path)
                .unwrap_or_else(|e| panic!("Cannot read {}: {}", expected_org_path.display(), e));
            if actual_org != expected_org {
                // Trim frontmatter if present and compare content
                let (expected_fm, expected_rest) = split_frontmatter(&expected_org);
                let (actual_fm, actual_rest) = split_frontmatter(&actual_org);
                match (expected_rest, actual_rest) {
                    (Some(expected_rest), Some(actual_rest)) => {
                        if expected_rest != actual_rest {
                            failures.push(format!("Content mismatch for {} (expected .org)", stem));
                        }
                    }
                    _ => {
                        failures.push(format!("Output mismatch for {} (expected .org)", stem));
                    }
                }
            }
        } else {
            // No expected file; just check that output contains Org syntax
            if !actual_org.contains("#+")
                && !actual_org.contains("* ")
                && !actual_org.contains("- ")
            {
                failures.push(format!(
                    "MDX→Org output for {} does not appear to be Org syntax (missing #+, *, or - markers)",
                    stem
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!("MDX→Org test failures:\n{}", failures.join("\n"));
    }
}

// Helper to split frontmatter for the new tests (reuses the same logic)
fn split_frontmatter(s: &str) -> (Option<Value>, Option<&str>) {
    let s = s.trim_start();
    if !s.starts_with("---\n") {
        return (None, Some(s));
    }
    let after_start = &s[4..];
    if let Some(end) = after_start.find("\n---\n") {
        let yaml_str = &after_start[..end];
        let rest = &after_start[end + 5..];
        match serde_yaml::from_str(yaml_str) {
            Ok(value) => (Some(value), Some(rest)),
            Err(_) => (None, Some(s)),
        }
    } else {
        (None, Some(s))
    }
}
