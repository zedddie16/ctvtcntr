use hyprland::data::Client;
use regex::Regex;
/// Normalizes the window title into a process name:
/// - Trims whitespace.
/// - If the title starts with "New Tab -", that prefix is removed (case‑insensitive).
/// - If a " | " separator is present, only the part before it is used.
pub fn process_complex_names(process_name: String, window: &Client) -> String {
    let rexex_str = Regex::new(r"^(.+?)\s*–\s*").unwrap(); // trims active title of app
    if window.class == "com.mitchellh.ghostty" {
        if window.title.contains("nvim") {
            return "NeoVim".to_string();
        } else {
            return "Ghostty".to_string();
        }
    }
    // Handle Kitty windows running nvim
    if window.class == "kitty" {
        if window.title.contains("nvim") {
            return "Vim".to_string();
        } else {
            return "Kitty".to_string();
        }
    }

    if window.class == "jetbrains-rustrover" && process_name.is_empty() {
        if let Some(cap) = rexex_str.captures(&window.title) {
            if let Some(extracted_match) = cap.get(1) {
                return format!("RustRover -> {}", extracted_match.as_str());
            }
        }
        return "RustRover".to_string();
    }

    process_name
}
pub fn extract_process_name(window_title: &str) -> String {
    let trimmed = &window_title.trim();

    match trimmed {
        t if t.contains("discord") => "Discord".to_string(),
        // t if t.contains("Obsidian") => "Obsidian".to_string(),
        t if t.contains("Telegram") => "Telegram".to_string(),
        t if t.contains("Sublime Text") => "Sublime Text".to_string(),
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
