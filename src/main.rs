use chrono::Local;
use csv::{ReaderBuilder, WriterBuilder};
use hyprland::data::Client;
use hyprland::shared::HyprDataActiveOptional;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    date: String,
    window_name: String,
    // usage time stored as a formatted string, e.g. "01h:23m:45s"
    total_time: String,
}

/// Converts a Duration into a formatted string "HHh:MMm:SSs".
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{:02}h:{:02}m:{:02}s", hours, minutes, seconds)
}

/// Parses a duration string in the format "HHh:MMm:SSs" back into a Duration.
/// TODO: rewrite conversion and deserealization
fn parse_duration_str(s: &str) -> Duration {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return Duration::ZERO;
    }
    let hours = parts[0].trim_end_matches('h').parse::<u64>().unwrap_or(0);
    let minutes = parts[1].trim_end_matches('m').parse::<u64>().unwrap_or(0);
    let seconds = parts[2].trim_end_matches('s').parse::<u64>().unwrap_or(0);
    Duration::from_secs(hours * 3600 + minutes * 60 + seconds)
}

/// Normalizes the window title into a process name:
/// - Trims whitespace.
/// - If the title starts with "New Tab -", that prefix is removed (case‑insensitive).
/// - If a " | " separator is present, only the part before it is used.
/// TODO: Remove manual check, improve Regex. (may be move these checks somwhere else as a big
/// match statement.)
fn extract_process_name(window_title: &str) -> String {
    let trimmed = window_title.trim();

    if trimmed.contains("Discord") {
        return "Discord".to_string();
    }
    if trimmed.contains("Telegram") {
        return "Telegram".to_string();
    }

    if trimmed.to_lowercase().starts_with("new tab -") {
        return trimmed["New Tab -".len()..].trim().to_string();
    }
    if trimmed.to_lowercase().starts_with(r#"win"#) {
        return "sub_window".to_string();
    }
    if let Some(idx) = trimmed.find(" | ") {
        return trimmed[..idx].trim().to_string();
    }
    trimmed.to_string()
}

/// Reads usage data from the CSV file into a HashMap keyed by (date, window_name).
/// If the file does not exist, returns an empty map.
fn read_usage_data(file_path: &str) -> io::Result<HashMap<(String, String), Duration>> {
    let mut usage_map = HashMap::new();
    if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        for result in csv_reader.deserialize() {
            let record: Usage = result?;
            // Normalize the process name (to merge similar entries).
            let key = (record.date, extract_process_name(&record.window_name));
            let dur = parse_duration_str(&record.total_time);
            usage_map
                .entry(key)
                .and_modify(|d| *d += dur)
                .or_insert(dur);
        }
    }
    Ok(usage_map)
}

/// Writes the current usage data to the CSV file.
/// Each record is stored as a row with date, window_name, and total_time (formatted).
fn write_usage_data(
    file_path: &str,
    usage_map: &HashMap<(String, String), Duration>,
) -> io::Result<()> {
    let file = File::create(file_path)?;
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(file);
    for ((date, window_name), duration) in usage_map {
        let record = Usage {
            date: date.clone(),
            window_name: window_name.clone(),
            total_time: format_duration(*duration),
        };
        csv_writer.serialize(record)?;
    }
    csv_writer.flush()?;
    Ok(())
}

/// Updates the usage map by adding elapsed time for the given (date, window_name) key.
fn update_usage(
    usage_map: &mut HashMap<(String, String), Duration>,
    date: &str,
    window_name: &str,
    elapsed: Duration,
) {
    let key = (date.to_string(), window_name.to_string());
    usage_map
        .entry(key)
        .and_modify(|d| *d += elapsed)
        .or_insert(elapsed);
}

/// Monitors the active window and updates usage data:
/// - Uses chrono to get the current date.
/// - If a record for the current date and process already exists, increments its usage time.
/// - Otherwise, creates a new record for today.
fn monitor_active_window(usage_map: &mut HashMap<(String, String), Duration>) -> io::Result<()> {
    let mut last_key: Option<(String, String)> = None; // (date, process name)
    let mut last_switch_time = Instant::now();

    let rexex_str = Regex::new(r"^(.+?)\s*–\s*").unwrap(); // trims active title of app

    info!("starting active window monitor loop");
    loop {
        let current_date = Local::now().format("%Y-%m-%d").to_string();
        if let Ok(Some(active_window)) = Client::get_active() {
            let raw_title = active_window.initial_title.clone();
            let mut process_name = extract_process_name(&raw_title);

            // Handle Kitty windows running nvim
            if active_window.class == "kitty" {
                if active_window.title.contains("nvim") {
                    process_name = "nvim".to_string();
                } else {
                    process_name = "kitty".to_string();
                }
            }

            if process_name.is_empty() {
                if !active_window.class.is_empty() {
                    process_name = active_window.class;
                    if process_name == *"jetbrains-rustrover" {
                        if let Some(captures) = rexex_str.captures(active_window.title.as_str()) {
                            let extracted = captures.get(1).unwrap().as_str();
                            process_name = format!("RustRover -> {}", extracted);
                        } else {
                            process_name = "RustRover".to_string();
                        }
                    }
                } else {
                    sleep(Duration::from_millis(50));
                    continue;
                }
            }
            let current_key = (current_date.clone(), process_name);
            if last_key.as_ref() != Some(&current_key) {
                if let Some(ref prev_key) = last_key {
                    let elapsed = last_switch_time.elapsed();
                    update_usage(usage_map, &prev_key.0, &prev_key.1, elapsed);
                }
                last_key = Some(current_key.clone());
                last_switch_time = Instant::now();
                info!("switched to: {}", current_key.1);
            }
        }

        // Write updated usage data to CSV.
        write_usage_data("app_usage.csv", usage_map)?;
        sleep(Duration::from_millis(50));
    }
}

fn main() -> io::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())
        .expect("setting default subscriber failed");
    // Load existing usage data (if any), then start monitoring.
    let mut usage_map = read_usage_data("app_usage.csv")?;
    info!("readed usage data");
    monitor_active_window(&mut usage_map)
}
