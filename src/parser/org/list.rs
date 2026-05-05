pub fn is_unordered_item(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("- ") || t.starts_with("+ ")
}

pub fn unordered_content(line: &str) -> &str {
    let t = line.trim();
    &t[2..]
}

pub fn is_ordered_item(line: &str) -> bool {
    let t = line.trim();
    let b = t.as_bytes();
    let mut i = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i < b.len() && b[i] == b'.' && i + 1 < b.len() && b[i + 1] == b' '
}

pub fn ordered_parts(line: &str) -> (&str, &str) {
    let t = line.trim();
    if let Some(dot) = t.find(". ") {
        (&t[..dot], &t[dot + 2..])
    } else {
        ("1", t)
    }
}
