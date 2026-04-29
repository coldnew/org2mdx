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

        let actual = match org2mdx::convert(&org_content) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("Conversion failed for {}: {}", stem, e));
                continue;
            }
        };

        if actual != expected {
            let actual_lines: Vec<&str> = actual.lines().collect();
            let expected_lines: Vec<&str> = expected.lines().collect();
            let max = actual_lines.len().max(expected_lines.len());
            let mut diff = String::new();
            for i in 0..max {
                let a = actual_lines.get(i).copied().unwrap_or("<missing>");
                let e = expected_lines.get(i).copied().unwrap_or("<missing>");
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
            failures.push(format!("Output mismatch for {}:{}", stem, diff));
        }
    }

    if !failures.is_empty() {
        panic!("Test failures:\n{}", failures.join("\n"));
    }
}
