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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub date: String,
    pub window_name: String,
    pub usage_time_secs: u32,
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
/// TODO: rewrite monitor active window logic to use duckdb
pub fn monitor_active_window(
    usage_map: &mut BTreeMap<(String, String), Duration>,
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
                }
                last_key = Some(current_key.clone());
                last_switch_time = Instant::now();
                info!("switched to: {}", current_key.1);
            }
        }

        // Write updated usage data to CSV.
        // write_usage_data("app_usage.csv", usage_map)?;
        sleep(Duration::from_millis(500));
    }
    info!("Shutting down");
    Ok(())
}
