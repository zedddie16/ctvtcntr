use crate::match_title::extract_process_name;
use chrono::Local;
use csv::{ReaderBuilder, WriterBuilder};
use hyprland::data::Client;
use hyprland::shared::HyprDataActiveOptional;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
// TODO: rewrite conversion and deserealization
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

/// Reads usage data from the CSV file into a HashMap keyed by (date, window_name).
/// If the file does not exist, returns an empty map.
pub fn read_usage_data(file_path: &PathBuf) -> io::Result<BTreeMap<(String, String), Duration>> {
    let mut usage_map = BTreeMap::new();
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
                .and_modify(|d| *d += dur) // if a record exists modify.
                .or_insert(dur); // if it doesn't, create new record.
        }
    }
    info!("readed usage data on: {file_path:?}");
    Ok(usage_map)
}

/// Writes the current usage data to the CSV file.
/// Each record is stored as a row with date, window_name, and total_time (formatted).
fn write_usage_data(
    usage_map: &BTreeMap<(String, String), Duration>,
    csv_path: &PathBuf,
) -> io::Result<()> {
    let file = File::create(csv_path)?;
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
    usage_map: &mut BTreeMap<(String, String), Duration>,
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
pub fn monitor_active_window(
    usage_map: &mut BTreeMap<(String, String), Duration>,
    csv_path: &PathBuf,
) -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true)); // creating app's state
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    let mut last_key: Option<(String, String)> = None; // (date, process name)
    let mut last_switch_time = Instant::now();

    let rexex_str = Regex::new(r"^(.+?)\s*–\s*").unwrap(); // trims active title of app

    info!("starting active window monitor loop");
    while running.load(Ordering::SeqCst) {
        let current_date = Local::now().format("%Y-%m-%d").to_string();
        if let Ok(Some(active_window)) = Client::get_active() {
            let raw_title = active_window.initial_title.clone();
            let mut process_name = extract_process_name(&raw_title);
            // if active_window.class == "obsidian" {
            //    process_name = *active_window.title.
            //    try_4 - Vault - Obsidian v1.8.10:
            // }
            if active_window.class == "com.mitchellh.ghostty" {
                if active_window.title.contains("nvim") {
                    process_name = "NeoVim".to_string();
                } else {
                    process_name = "Ghostty".to_string();
                }
            }
            // Handle Kitty windows running nvim
            if active_window.class == "kitty" {
                if active_window.title.contains("nvim") {
                    process_name = "NeoVim".to_string();
                } else {
                    process_name = "Kitty".to_string();
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
                    sleep(Duration::from_millis(500));
                    continue;
                }
            }
            let current_key = (current_date.clone(), process_name);
            if last_key.as_ref() != Some(&current_key) {
                if let Some(ref prev_key) = last_key {
                    let elapsed = last_switch_time.elapsed();
                    update_usage(usage_map, &prev_key.0, &prev_key.1, elapsed);
                    // Write updated usage data to CSV.
                    write_usage_data(usage_map, csv_path)?;
                    info!("Written to {csv_path:?}");
                }
                last_key = Some(current_key.clone());
                last_switch_time = Instant::now();
                info!("switched to: {}", current_key.1);
            }
        }

        sleep(Duration::from_millis(500));
    }
    if let Some(ref final_key) = last_key {
        let elapsed_final = last_switch_time.elapsed();
        update_usage(usage_map, &final_key.0, &final_key.1, elapsed_final);
        if let Err(e) = write_usage_data(usage_map, csv_path) {
            tracing::error!("Failed to write final usage data on shutdown: {}", e);
        } else {
            info!("Final usage data successfully written to {:?}", csv_path);
        }
    }
    info!("Shutting down application");
    Ok(())
}
