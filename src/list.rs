pub fn is_unordered_item(line: &str) -> bool {
    let t = line.trim_start();
    (t.starts_with("- ") || t.starts_with("+ ")) && t.len() > 2
}

pub fn unordered_content(line: &str) -> &str {
    let t = line.trim_start();
    &t[2..]
}

pub fn is_ordered_item(line: &str) -> bool {
    let t = line.trim_start();
    let b = t.as_bytes();
    let mut i = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && b.get(i) == Some(&b'.') && b.get(i + 1) == Some(&b' ')
}

pub fn ordered_parts(line: &str) -> (&str, &str) {
    let t = line.trim_start();
    if let Some(dot) = t.find(". ") {
        (&t[..dot], &t[dot + 2..])
    } else {
        ("1", t)
    }
}
