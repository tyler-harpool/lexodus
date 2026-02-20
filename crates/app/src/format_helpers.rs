/// Shared formatting utilities for the UI layer.
///
/// All functions accept ISO-8601 date strings (e.g. "2026-01-20T21:35:00Z")
/// and produce human-readable output without external crate dependencies.

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Parse month number (1-12) from a two-digit string.
fn parse_month(s: &str) -> Option<usize> {
    s.parse::<usize>().ok().filter(|m| (1..=12).contains(m))
}

/// Format an ISO date string as "Jan 20, 2026" (date-only, human-readable).
///
/// Falls back to the first 10 characters if parsing fails.
pub fn format_date_human(date_str: &str) -> String {
    if date_str.len() < 10 {
        return date_str.to_string();
    }
    let year = &date_str[..4];
    let month = &date_str[5..7];
    let day = &date_str[8..10];

    if let Some(m) = parse_month(month) {
        let day_num: u32 = day.parse().unwrap_or(0);
        format!("{} {}, {}", MONTH_NAMES[m - 1], day_num, year)
    } else {
        date_str[..10].to_string()
    }
}

/// Format an ISO datetime string as "Jan 20, 2026 9:35 PM" (with 12-hour time).
///
/// Falls back to date-only if time portion is missing.
pub fn format_datetime_human(date_str: &str) -> String {
    let date_part = format_date_human(date_str);

    // Need at least "YYYY-MM-DDTHH:MM" (16 chars)
    if date_str.len() < 16 {
        return date_part;
    }

    let hour_str = &date_str[11..13];
    let min_str = &date_str[14..16];

    let hour: u32 = match hour_str.parse() {
        Ok(h) => h,
        Err(_) => return date_part,
    };

    let (display_hour, ampm) = match hour {
        0 => (12, "AM"),
        1..=11 => (hour, "AM"),
        12 => (12, "PM"),
        _ => (hour - 12, "PM"),
    };

    format!("{} {}:{} {}", date_part, display_hour, min_str, ampm)
}

/// Convert a snake_case string to Title Case (e.g. "status_conference" → "Status Conference").
pub fn format_snake_case_title(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
