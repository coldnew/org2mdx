use crate::ast::Node;

/// Iterates over frontmatter keys in order, then remaining keys alphabetically,
/// calling `render_kv` for each key-value pair found in the root node's data map.
pub fn render_frontmatter<F>(out: &mut String, root: &Node, ordered_keys: &[&str], mut render_kv: F)
where
    F: FnMut(&mut String, &str, &serde_json::Value),
{
    for key in ordered_keys {
        if let Some(value) = root.data.get(*key) {
            render_kv(out, key, value);
        }
    }
    let mut remaining_keys: Vec<&String> = root
        .data
        .keys()
        .filter(|k| {
            let k_lower = k.to_lowercase();
            !ordered_keys.contains(&k_lower.as_str())
        })
        .collect();
    remaining_keys.sort();
    for key in remaining_keys {
        let value = &root.data[key];
        render_kv(out, key, value);
    }
}
