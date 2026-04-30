pub fn collapse_spaces(s: &str) -> String {
    let mut result = s.to_string();
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    result
}

pub fn kw<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let prefix = format!("#+{}:", keyword.to_uppercase());
    let lp = format!("#+{}:", keyword.to_lowercase());
    let t = line.trim_start();
    let tu = t.to_uppercase();
    if tu.starts_with(&prefix) {
        let rest = &t[prefix.len()..];
        Some(rest.trim())
    } else if t.starts_with(&lp) {
        Some(t[lp.len()..].trim())
    } else {
        None
    }
}

pub fn strip_prefix_spaces(s: &str, n: usize) -> &str {
    let mut count = 0;
    let b = s.as_bytes();
    while count < n && count < b.len() && (b[count] == b' ' || b[count] == b'\t') {
        count += 1;
    }
    &s[count..]
}

pub fn pct_encode(path: &str) -> String {
    let mut out = String::new();
    for c in path.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '+') {
            out.push(c);
        } else {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            for b in s.bytes() {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

pub fn escape_url_parens(url: &str) -> String {
    url.replace('(', "\\(")
        .replace(')', "\\)")
        .replace('&', "\\&")
}

pub fn yaml_str(s: &str) -> String {
    let needs_quote = s.contains('"')
        || s.contains(':')
        || s.starts_with('\'')
        || s.starts_with('{')
        || s.starts_with('[');
    if needs_quote {
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

// ============================================================
// HTML ↔ JSX Conversion
// ============================================================

/// Void (self-closing) HTML elements that require `/>` in JSX
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// HTML attribute name → JSX attribute name
const ATTR_TO_JSX: &[(&str, &str)] = &[
    ("class", "className"),
    ("for", "htmlFor"),
    ("tabindex", "tabIndex"),
    ("onclick", "onClick"),
    ("onchange", "onChange"),
    ("onsubmit", "onSubmit"),
    ("onfocus", "onFocus"),
    ("onblur", "onBlur"),
    ("onkeydown", "onKeyDown"),
    ("onkeyup", "onKeyUp"),
    ("onload", "onLoad"),
    ("onerror", "onError"),
    ("readonly", "readOnly"),
    ("maxlength", "maxLength"),
];

/// JSX attribute name → HTML attribute name (reverse)
const ATTR_TO_HTML: &[(&str, &str)] = &[
    ("className", "class"),
    ("htmlFor", "for"),
    ("tabIndex", "tabindex"),
    ("onClick", "onclick"),
    ("onChange", "onchange"),
    ("onSubmit", "onsubmit"),
    ("onFocus", "onfocus"),
    ("onBlur", "onblur"),
    ("onKeyDown", "onkeydown"),
    ("onKeyUp", "onkeyup"),
    ("onLoad", "onload"),
    ("onError", "onerror"),
    ("readOnly", "readonly"),
    ("maxLength", "maxlength"),
];

pub fn html_to_jsx(html: &str) -> String {
    let mut result = html.to_string();
    result = convert_html_comments(&result);
    result = convert_void_elements_to_jsx(&result);
    result = convert_attrs(&result, ATTR_TO_JSX);
    result = convert_style_to_jsx(&result);
    result
}

pub fn jsx_to_html(jsx: &str) -> String {
    let mut result = jsx.to_string();
    result = convert_jsx_comments(&result);
    result = convert_void_elements_to_html(&result);
    result = convert_attrs(&result, ATTR_TO_HTML);
    result = convert_style_to_html(&result);
    result
}

/// Convert all HTML comments `<!-- ... -->` to JSX `{/* ... */}`
fn convert_html_comments(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;
    while let Some(start) = remaining.find("<!--") {
        result.push_str(&remaining[..start]);
        let after = &remaining[start + 4..];
        match after.find("-->") {
            Some(end) => {
                let inner = after[..end].trim();
                result.push_str(&format!("{{/* {} */}}", inner));
                remaining = &after[end + 3..];
            }
            None => {
                result.push_str("<!--");
                remaining = after;
            }
        }
    }
    result.push_str(remaining);
    result
}

/// Convert all JSX comments `{/* ... */}` to HTML `<!-- ... -->`
fn convert_jsx_comments(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;
    while let Some(start) = remaining.find("{/*") {
        result.push_str(&remaining[..start]);
        let after = &remaining[start + 3..];
        match after.find("*/}") {
            Some(end) => {
                let inner = after[..end].trim();
                result.push_str(&format!("<!-- {} -->", inner));
                remaining = &after[end + 3..];
            }
            None => {
                result.push_str("{/*");
                remaining = after;
            }
        }
    }
    result.push_str(remaining);
    result
}

/// Convert void HTML elements to self-closing JSX form
fn convert_void_elements_to_jsx(s: &str) -> String {
    process_tags(s, |tag_name, attrs, closing| {
        let tag_lower = tag_name.to_lowercase();
        if VOID_ELEMENTS.contains(&tag_lower.as_str()) {
            if closing.is_empty() {
                format!("<{}{} />", tag_name, attrs)
            } else {
                format!("<{}{} />", tag_name, attrs.trim_end_matches('/'))
            }
        } else {
            format!("<{}{}{}>", tag_name, attrs, closing)
        }
    })
}

/// Convert self-closing void JSX elements back to HTML form
fn convert_void_elements_to_html(s: &str) -> String {
    process_tags(s, |tag_name, attrs, _closing| {
        let tag_lower = tag_name.to_lowercase();
        if VOID_ELEMENTS.contains(&tag_lower.as_str()) {
            format!("<{}{}>", tag_name, attrs.trim_end_matches('/').trim_end())
        } else {
            format!("<{}{}>", tag_name, attrs)
        }
    })
}

/// Process each HTML/JSX tag in the string, applying a transformation.
/// The callback receives: (tag_name, attributes_string, closing_marker)
/// closing_marker is `/>`, `>`, or empty string
fn process_tags<F>(s: &str, transform: F) -> String
where
    F: Fn(&str, &str, &str) -> String,
{
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;
    while let Some(lt) = remaining.find('<') {
        result.push_str(&remaining[..lt]);
        let after_lt = &remaining[lt..];
        if after_lt.len() < 2 {
            result.push('<');
            remaining = &after_lt[1..];
            continue;
        }
        let next = after_lt.as_bytes()[1];
        if next == b'/' || next == b'!' || next == b'?' {
            if let Some(gt) = after_lt.find('>') {
                result.push_str(&after_lt[..=gt]);
                remaining = &after_lt[gt + 1..];
            } else {
                result.push_str(after_lt);
                remaining = "";
            }
            continue;
        }
        if let Some(gt) = find_tag_end(after_lt) {
            let full_tag = &after_lt[..=gt];
            let tag_inner = &full_tag[1..full_tag.len() - 1];
            let is_self_closing = tag_inner.ends_with('/');
            let tag_body = if is_self_closing {
                &tag_inner[..tag_inner.len() - 1]
            } else {
                tag_inner
            };
            let name_end = tag_body
                .find(|c: char| c.is_ascii_whitespace())
                .unwrap_or(tag_body.len());
            let tag_name = &tag_body[..name_end];
            let attrs = &tag_body[name_end..];
            let new_tag = transform(tag_name, attrs, if is_self_closing { "/>" } else { ">" });
            result.push_str(&new_tag);
            remaining = &after_lt[gt + 1..];
        } else {
            result.push('<');
            remaining = &after_lt[1..];
        }
    }
    result.push_str(remaining);
    result
}

/// Find the matching `>` for a tag starting with `<`.
/// Handles quoted attribute values.
fn find_tag_end(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    for (i, &b) in bytes.iter().enumerate().skip(1) {
        if in_single {
            if b == b'\'' {
                in_single = false;
            }
        } else if in_double {
            if b == b'"' {
                in_double = false;
            }
        } else {
            match b {
                b'\'' => in_single = true,
                b'"' => in_double = true,
                b'>' => return Some(i),
                _ => {}
            }
        }
    }
    None
}

/// Replace attribute names within HTML/JSX tags.
fn convert_attrs(s: &str, map: &[(&str, &str)]) -> String {
    if map.is_empty() {
        return s.to_string();
    }
    process_tags(s, |tag_name, attrs, closing| {
        let mut new_attrs = attrs.to_string();
        for (from, to) in map {
            let from_eq = format!("{}=", from);
            let to_eq = format!("{}=", to);
            let space_from = format!(" {}", &from_eq);
            new_attrs = new_attrs.replace(&space_from, &format!(" {}", &to_eq));
            if new_attrs.trim_start().starts_with(&from_eq) {
                new_attrs = new_attrs.replacen(&from_eq, &to_eq, 1);
            }
        }
        format!("<{}{}{}>", tag_name, new_attrs, closing)
    })
}

/// Convert HTML style attribute to JSX style object
fn convert_style_to_jsx(s: &str) -> String {
    process_tags(s, |tag_name, attrs, closing| {
        let new_attrs = replace_style_value(attrs, |value| {
            let props: Vec<String> = value
                .split(';')
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
                .filter_map(|p| {
                    let mut parts = p.splitn(2, ':');
                    let key = parts.next()?.trim();
                    let val = parts.next()?.trim();
                    let camel = kebab_to_camel(key);
                    Some(format!("{}: '{}'", camel, val))
                })
                .collect();
            if props.is_empty() {
                String::new()
            } else {
                format!("{{{{{}}}}}", props.join(", "))
            }
        });
        format!("<{}{}{}>", tag_name, new_attrs, closing)
    })
}

/// Convert JSX style object back to HTML style string
fn convert_style_to_html(s: &str) -> String {
    process_tags(s, |tag_name, attrs, closing| {
        let new_attrs = replace_style_value(attrs, |value| {
            let inner = value.trim();
            if inner.starts_with("{{") && inner.ends_with("}}") {
                let obj = &inner[2..inner.len() - 2];
                let props: Vec<String> = obj
                    .split(',')
                    .filter_map(|p| {
                        let mut parts = p.splitn(2, ':');
                        let key = parts.next()?.trim();
                        let val = parts.next()?.trim().trim_matches('\'').trim_matches('"');
                        let kebab = camel_to_kebab(key);
                        Some(format!("{}: {}", kebab, val))
                    })
                    .collect();
                props.join("; ")
            } else {
                inner.to_string()
            }
        });
        format!("<{}{}{}>", tag_name, new_attrs, closing)
    })
}

/// Find and replace the value of a `style=` attribute in an attributes string
fn replace_style_value<F>(attrs: &str, f: F) -> String
where
    F: Fn(&str) -> String,
{
    let style_eq = "style=";
    if let Some(pos) = attrs.find(style_eq) {
        let after_eq = &attrs[pos + style_eq.len()..];
        let (value, rest) = extract_attr_value(after_eq);
        let new_value = f(value);
        format!("{}style=\"{}\"{}", &attrs[..pos], new_value, rest)
    } else {
        attrs.to_string()
    }
}

/// Extract an attribute value (quoted or unquoted)
/// Returns (value, rest_of_string)
fn extract_attr_value(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    if s.is_empty() {
        return ("", "");
    }
    let bytes = s.as_bytes();
    if bytes[0] == b'"' || bytes[0] == b'\'' {
        let quote = bytes[0] as char;
        if let Some(end) = s[1..].find(quote) {
            (&s[1..end + 1], &s[end + 2..])
        } else {
            ("", s)
        }
    } else {
        let end = s
            .find(|c: char| c.is_ascii_whitespace() || c == '>')
            .unwrap_or(s.len());
        (&s[..end], &s[end..])
    }
}

/// Convert kebab-case CSS property to camelCase
fn kebab_to_camel(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut upper = false;
    for c in s.chars() {
        if c == '-' {
            upper = true;
        } else if upper {
            result.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert camelCase to kebab-case
fn camel_to_kebab(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            result.push('-');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

pub fn org_date_to_iso(s: &str) -> Option<String> {
    let s = s.trim().trim_start_matches('<').trim_end_matches('>');
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let date_str = parts[0];
    let time_str = parts
        .iter()
        .find(|p| p.contains(':'))
        .copied()
        .unwrap_or("00:00");

    let dp: Vec<&str> = date_str.split('-').collect();
    if dp.len() != 3 {
        return None;
    }
    let y: i32 = dp[0].parse().ok()?;
    let mo: u32 = dp[1].parse().ok()?;
    let d: u32 = dp[2].parse().ok()?;

    let tp: Vec<&str> = time_str.split(':').collect();
    let h: i32 = tp.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let m: i32 = tp.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    let mut utc_h = h - 8;
    let mut utc_d = d as i32;
    let mut utc_mo = mo;
    let mut utc_y = y;

    if utc_h < 0 {
        utc_h += 24;
        utc_d -= 1;
        if utc_d < 1 {
            if utc_mo == 1 {
                utc_mo = 12;
                utc_y -= 1;
            } else {
                utc_mo -= 1;
            }
            utc_d = days_in_month(utc_y, utc_mo) as i32;
        }
    }

    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:00.000Z",
        utc_y, utc_mo, utc_d, utc_h, m
    ))
}

pub fn iso_to_org_date(s: &str) -> Option<String> {
    let s = s.trim();
    let (date_part, time_part) = if let Some(idx) = s.find('T') {
        (&s[..idx], &s[idx + 1..])
    } else {
        return None;
    };
    let dp: Vec<&str> = date_part.split('-').collect();
    if dp.len() != 3 {
        return None;
    }
    let y: i32 = dp[0].parse().ok()?;
    let mo: u32 = dp[1].parse().ok()?;
    let d: u32 = dp[2].parse().ok()?;

    let tp: Vec<&str> = time_part.split(':').collect();
    let utc_h: i32 = tp.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let utc_m: i32 = tp.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    let mut local_h = utc_h + 8;
    let mut local_d = d as i32;
    let mut local_mo = mo;
    let mut local_y = y;

    if local_h >= 24 {
        local_h -= 24;
        local_d += 1;
        if local_d > days_in_month(local_y, local_mo) as i32 {
            local_d = 1;
            if local_mo == 12 {
                local_mo = 1;
                local_y += 1;
            } else {
                local_mo += 1;
            }
        }
    }

    let wd = {
        let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y_adj = if local_mo < 3 { local_y - 1 } else { local_y };
        let d = (y_adj + y_adj / 4 - y_adj / 100
            + y_adj / 400
            + t[local_mo as usize - 1]
            + local_d as i32)
            % 7;
        match d {
            0 => "Sun",
            1 => "Mon",
            2 => "Tue",
            3 => "Wed",
            4 => "Thu",
            5 => "Fri",
            _ => "Sat",
        }
    };

    Some(format!(
        "<{:04}-{:02}-{:02} {} {:02}:{:02}>",
        local_y, local_mo, local_d, wd, local_h, utc_m
    ))
}

pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}
