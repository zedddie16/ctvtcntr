// TODO: Remove manual check, improve Regex. (may be move these checks somwhere else as a big match statement.)

/// Normalizes the window title into a process name:
/// - Trims whitespace.
/// - If the title starts with "New Tab -", that prefix is removed (case‑insensitive).
/// - If a " | " separator is present, only the part before it is used.
pub fn extract_process_name(window_title: &str) -> String {
    let trimmed = &window_title.trim();

    match trimmed {
        t if t.contains("discord") => "Discord".to_string(),
        t if t.contains("Telegram") => "Telegram".to_string(),
        t if t.starts_with("new tab -") => t["new tab -".len()..].trim().to_string(),
        t if t.starts_with("win") => "sub_window".to_string(),
        _ => trimmed.to_string(),
    }
    // if let Some(idx) = trimmed.find(" | ") {
    //     trimmed[..idx].trim().to_string()
    // } else {
    //     trimmed.to_string()
    // }
}
