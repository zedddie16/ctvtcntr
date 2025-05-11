use duckdb::{params, Connection, Result};
use std::io;
use tracing::{error, info};

mod db;
use db::{self, ensure_table_exists};
mod logic;
use logic::{monitor_active_window, read_usage_data};
mod startup;
use startup::wait_for_hyprland_socket;
mod match_title;

fn main() -> io::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())
        .expect("setting default subscriber failed");

    // wait for hyprland IPC
    if let Err(e) = wait_for_hyprland_socket(60) {
        // Wait for up to 60 seconds
        error!("Error waiting for Hyprland: {}", e);
        return Err(io::Error::new(io::ErrorKind::Other, e));
    }
    let conn = Connection::open("records.db")?;
    info!("connected to duckdb");
    // Load existing usage data (if any), then start monitoring.
    ensure_table_exists(conn);
    info!("Table 'activity_log' ensured.");
    let mut usage_map = read_usage_data("app_usage.csv")?;
    info!("readed usage data");
    monitor_active_window(&mut usage_map)
}
