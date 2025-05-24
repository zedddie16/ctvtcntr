use std::io;
use tracing::{error, info};

mod logic;
use logic::{monitor_active_window, read_usage_data};
mod startup;
use startup::wait_for_hyprland_socket;
mod match_title;

use std::env;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())
        .expect("setting default subscriber failed");

    // wait for hyprland IPC
    if let Err(e) = wait_for_hyprland_socket(60) {
        // Wait for up to 60 seconds
        error!("Error waiting for Hyprland: {}", e);
        return Err(io::Error::new(io::ErrorKind::Other, e));
    }

    // Load data path exposed by build script
    let data_dir_str = env!("CTVTCNTR_DATA_DIR");
    let data_dir = PathBuf::from(data_dir_str);
    let app_usage_file_path = data_dir.join("app_usage.csv");

    // Load existing usage data (if any), then start monitoring.
    let mut usage_map = read_usage_data(&app_usage_file_path)?;
    monitor_active_window(&mut usage_map)
}
