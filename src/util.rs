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
