use csv::ReaderBuilder;
use hyprland::data::Client;
use hyprland::shared::HyprDataActiveOptional;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::thread::sleep;
use std::time::{Duration, Instant};

use csv::WriterBuilder;
use std::io::Write;

fn write_usage_data(file_path: &str, usage_map: &HashMap<String, Duration>) -> io::Result<()> {
    let file = File::create(file_path)?;
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(file);

    for (app_name, total_time) in usage_map {
        let record = Usage {
            window_name: app_name.clone(),
            total_time: *total_time,
        };
        csv_writer.serialize(record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
struct Usage {
    window_name: String,
    total_time: Duration,
}

fn update_usage(usage_map: &mut HashMap<String, Duration>, app_name: &str, time_spent: Duration) {
    usage_map
        .entry(app_name.to_string())
        .and_modify(|e| *e += time_spent)
        .or_insert(time_spent);
}
fn read_usage_data(file_path: &str) -> io::Result<HashMap<String, Duration>> {
    let mut usage_map = HashMap::new();
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    for result in csv_reader.deserialize() {
        let record: Usage = result?;
        usage_map.insert(record.window_name, record.total_time);
    }

    Ok(usage_map)
}
fn monitor_active_window() {
    let mut window_times: HashMap<String, Duration> = HashMap::new();
    let mut last_window_title: Option<String> = None;
    let mut last_switch_time = Instant::now();

    loop {
        if let Ok(Some(active_window)) = Client::get_active() {
            let window_title = active_window.initial_title.clone();

            if last_window_title.as_ref() != Some(&window_title) {
                if let Some(prev_title) = &last_window_title {
                    let elapsed = last_switch_time.elapsed();
                    *window_times
                        .entry(prev_title.clone())
                        .or_insert(Duration::ZERO) += elapsed;
                }

                last_window_title = Some(window_title.clone());
                last_switch_time = Instant::now();

                println!("Switched to: {} at {:?}", window_title, last_switch_time);
            }
            sleep(Duration::from_millis(500));
        }
    }
}
fn main() {
    monitor_active_window()
}

