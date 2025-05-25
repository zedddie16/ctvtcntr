use duckdb::Connection;
use tracing::{error, info};

mod db;
use db::{ensure_table_exists, print_all_records};
mod logic;
use logic::monitor_active_window;
mod startup;
use startup::wait_for_hyprland_socket;
mod match_title;

mod utils;
use std::env;
use std::path::PathBuf;
use utils::get_app_data_dir;

fn main() {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())
        .expect("setting default subscriber failed");

    // wait for hyprland IPC
    if let Err(e) = wait_for_hyprland_socket(60) {
        error!("Error waiting for Hyprland: {}", e);
    }
    let path_to_database_file = match get_app_data_dir() {
        Ok(mut app_data_dir) => {
            app_data_dir.push("records.db");
            app_data_dir
        }
        Err(e) => {
            error!("Failed to get_app_data_dir: {}", e);
            panic!("Critical error: Could not set up application data directory.");
        }
    };
    let conn = Connection::open(&path_to_database_file).unwrap();
    info!("connected to duckdb on {path_to_database_file:?}");
    ensure_table_exists(&conn).expect("ensuring failed");
    info!("Table 'activity_log' ensured.");
    print_all_records(&conn);
    monitor_active_window(conn).expect("failed to start active monitor window loop");
}
