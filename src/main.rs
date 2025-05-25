use std::io;
use tracing::error;

mod logic;
use logic::{monitor_active_window, read_usage_data};
mod startup;
use startup::wait_for_hyprland_socket;
mod match_title;

mod utils;
use utils::get_app_data_dir;

fn main() -> io::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())
        .expect("setting default subscriber failed");

    // wait for hyprland IPC
    if let Err(e) = wait_for_hyprland_socket(60) {
        // Wait for up to 60 seconds
        error!("Error waiting for Hyprland: {}", e);
        return Err(io::Error::new(io::ErrorKind::Other, e));
    }
    let app_usage_file_path = match get_app_data_dir() {
        Ok(mut app_data_dir) => {
            app_data_dir.push("app_usage.csv");
            app_data_dir
        }
        Err(e) => {
            error!("Failed to get_app_data_dir: {}", e);
            panic!("Critical error: Could not set up application data directory.");
        }
    };
    // Load existing usage data (if any), then start monitoring.
    let mut usage_map = match read_usage_data(&app_usage_file_path) {
        Ok(usage_map) => usage_map,
        Err(e) => {
            error!("failed to read_usage_data: {e}");
            panic!("Critical error: Could not read app_usage.csv properly, aborting.");
        }
    };
    monitor_active_window(&mut usage_map, &app_usage_file_path)
}
