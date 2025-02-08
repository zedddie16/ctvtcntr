use hyprland::data::Client;
use hyprland::shared::HyprDataActiveOptional; // brings get_active() into scope
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
    #[serde(default)]
    total_time_secs: u64,
}

/// Normalizes the window title into a process name:
/// - Removes any leading/trailing whitespace.
/// - If the title starts with "New Tab -", that prefix is removed.
/// - If there's a " | " separator, only the part before the pipe is used.
/// - Otherwise, returns the trimmed title.
fn extract_process_name(window_title: &str) -> String {
    let trimmed = window_title.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Remove "New Tab -" prefix (case-insensitive)
    if trimmed.to_lowercase().starts_with("new tab -") {
        return trimmed["New Tab -".len()..].trim().to_string();
    }
    // If there's a pipe separator, use only the part before the pipe.
    if let Some(idx) = trimmed.find(" | ") {
        return trimmed[..idx].trim().to_string();
    }
    trimmed.to_string()
}

/// Reads usage data from a CSV file, normalizing process names in the process.
/// If the file doesn't exist, returns an empty map.
fn read_usage_data(file_path: &str) -> io::Result<HashMap<String, Duration>> {
    let mut usage_map = HashMap::new();
    if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        for result in csv_reader.deserialize() {
            let record: Usage = result?;
            let normalized = extract_process_name(&record.window_name);
            if !normalized.is_empty() {
                usage_map
                    .entry(normalized)
                    .and_modify(|d| *d += Duration::from_secs(record.total_time_secs))
                    .or_insert(Duration::from_secs(record.total_time_secs));
            }
        }
    }
    Ok(usage_map)
}

/// Writes the current usage data to a CSV file.
fn write_usage_data(file_path: &str, usage_map: &HashMap<String, Duration>) -> io::Result<()> {
    let file = File::create(file_path)?;
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(file);
    for (window_name, total_time) in usage_map {
        let record = Usage {
            window_name: window_name.clone(),
            total_time_secs: total_time.as_secs(),
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
        .and_modify(|e| *e += time_spent)
        .or_insert(time_spent);
}

/// Monitors the active window, updates the usage map, and writes data to CSV.
/// The window title is normalized before updating the record.
fn monitor_active_window(usage_map: &mut HashMap<String, Duration>) -> io::Result<()> {
    let mut last_window_name: Option<String> = None;
    let mut last_switch_time = Instant::now();

    loop {
        // Fetch the current active window (get_active takes no arguments).
        if let Ok(Some(active_window)) = Client::get_active() {
            let raw_title = active_window.initial_title.clone();
            let process_name = extract_process_name(&raw_title);
            // Skip if the normalized process name is empty.
            if process_name.is_empty() {
                sleep(Duration::from_millis(500));
                continue;
            }
            // If the active process has changed, update the previous record.
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
        // Periodically write the updated usage data to the CSV file.
        write_usage_data("app_usage.csv", usage_map)?;
        sleep(Duration::from_millis(500));
    }
}

fn main() -> io::Result<()> {
    // Load existing usage data (normalizing process names) or start with an empty map.
    let mut usage_map = read_usage_data("app_usage.csv")?;
    monitor_active_window(&mut usage_map)
}
