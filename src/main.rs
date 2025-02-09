use hyprland::data::Client;
use hyprland::shared::HyprDataActiveOptional; // Brings get_active() into scope
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::thread::sleep;
use std::time::{Duration, Instant};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    window_name: String,
    // total_time is stored as a formatted string, e.g., "02h:30m:15s"
    total_time: String,
}

/// Converts a Duration to a human‑friendly string in hours, minutes, and seconds.
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{:02}h:{:02}m:{:02}s", hours, minutes, seconds)
}

/// Parses a duration string in the format "HHh:MMm:SSs" to a Duration.
fn parse_duration_str(s: &str) -> Duration {
    // Split the string by ':' expecting 3 parts: "HHh", "MMm", "SSs"
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return Duration::ZERO;
    }
    let hours_str = parts[0].trim_end_matches('h');
    let minutes_str = parts[1].trim_end_matches('m');
    let seconds_str = parts[2].trim_end_matches('s');
    let hours: u64 = hours_str.parse().unwrap_or(0);
    let minutes: u64 = minutes_str.parse().unwrap_or(0);
    let seconds: u64 = seconds_str.parse().unwrap_or(0);
    Duration::from_secs(hours * 3600 + minutes * 60 + seconds)
}

/// Normalizes the window title into a process name:
/// - Trims whitespace.
/// - Removes a "New Tab -" prefix (if present, case‑insensitive).
/// - If a " | " separator is present, only the first part is used.
fn extract_process_name(window_title: &str) -> String {
    let trimmed = window_title.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Remove "New Tab -" prefix (case-insensitive)
    if trimmed.to_lowercase().starts_with("new tab -") {
        return trimmed["New Tab -".len()..].trim().to_string();
    }
    // If there's a pipe separator, take only the part before it.
    if let Some(idx) = trimmed.find(" | ") {
        return trimmed[..idx].trim().to_string();
    }
    trimmed.to_string()
}

/// Reads usage data from a CSV file, normalizing process names.
/// If the file does not exist, returns an empty map.
fn read_usage_data(file_path: &str) -> io::Result<HashMap<String, Duration>> {
    let mut usage_map = HashMap::new();
    if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        for result in csv_reader.deserialize() {
            let record: Usage = result?;
            let duration = parse_duration_str(&record.total_time);
            let normalized = extract_process_name(&record.window_name);
            if !normalized.is_empty() {
                usage_map
                    .entry(normalized)
                    .and_modify(|d| *d += duration)
                    .or_insert(duration);
            }
        }
    }
    Ok(usage_map)
}

/// Writes the current usage data to a CSV file,
/// storing the time as a formatted "HHh:MMm:SSs" string.
fn write_usage_data(file_path: &str, usage_map: &HashMap<String, Duration>) -> io::Result<()> {
    let file = File::create(file_path)?;
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(file);
    for (window_name, duration) in usage_map {
        let record = Usage {
            window_name: window_name.clone(),
            total_time: format_duration(*duration),
        };
        csv_writer.serialize(record)?;
    }
    csv_writer.flush()?;
    Ok(())
}

/// Updates the usage map with additional time for a given process name.
fn update_usage(usage_map: &mut HashMap<String, Duration>, window_name: &str, time_spent: Duration) {
    usage_map
        .entry(window_name.to_string())
        .and_modify(|d| *d += time_spent)
        .or_insert(time_spent);
}

/// Monitors the active window, updates the usage map,
/// writes data to the CSV file, and prints a summary every 30 seconds.
fn monitor_active_window(usage_map: &mut HashMap<String, Duration>) -> io::Result<()> {
    let mut last_window_name: Option<String> = None;
    let mut last_switch_time = Instant::now();
    let mut last_summary_time = Instant::now();

    loop {
        if let Ok(Some(active_window)) = Client::get_active() {
            let raw_title = active_window.initial_title.clone();
            let process_name = extract_process_name(&raw_title);
            if process_name.is_empty() {
                sleep(Duration::from_millis(500));
                continue;
            }
            if last_window_name.as_ref() != Some(&process_name) {
                if let Some(prev_name) = &last_window_name {
                    let elapsed = last_switch_time.elapsed();
                    update_usage(usage_map, prev_name, elapsed);
                }
                last_window_name = Some(process_name.clone());
                last_switch_time = Instant::now();
                println!("Switched to: {} at {:?}", process_name, last_switch_time);
            }
        }

        write_usage_data("app_usage.csv", usage_map)?;

        if last_summary_time.elapsed() >= Duration::from_secs(30) {
            println!("Usage summary:");
            for (name, duration) in usage_map.iter() {
                println!("{} - {}", name, format_duration(*duration));
            }
            last_summary_time = Instant::now();
        }

        sleep(Duration::from_millis(500));
    }
}

fn main() -> io::Result<()> {
    // Load existing usage data (or start with an empty map)
    let mut usage_map = read_usage_data("app_usage.csv")?;
    monitor_active_window(&mut usage_map)
}
