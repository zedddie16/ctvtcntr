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

#[derive(Deserialize, Serialize, Debug)]
struct Usage {
    window_name: String,
    total_time: Duration,
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
fn main() {
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
        }

        sleep(Duration::from_millis(500));
    }
}

