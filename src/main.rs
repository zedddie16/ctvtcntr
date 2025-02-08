
use hyprland::shared::HyprDataActiveOptional;
use hyprland::data::{self, Client};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread::sleep;

fn main() {
    let mut window_times: HashMap<String, Duration> = HashMap::new();
    let mut last_window_title: Option<String> = None;
    let mut last_switch_time = Instant::now();

    loop {
        if let Ok(Some(active_window)) = Client::get_active() {
            let window_title = active_window.title.clone();

            if last_window_title.as_ref() != Some(&window_title) {
                if let Some(prev_title) = &last_window_title {
                    let elapsed = last_switch_time.elapsed();
                    *window_times.entry(prev_title.clone()).or_insert(Duration::ZERO) += elapsed;
                }

                last_window_title = Some(window_title.clone());
                last_switch_time = Instant::now();

                println!("Switched to: {} at {:?}", window_title, last_switch_time);
            }
        }

        sleep(Duration::from_millis(500));
    }
}