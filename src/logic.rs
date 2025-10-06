use crate::db::log_activity;
use crate::match_title::extract_process_name;
use crate::match_title::process_complex_names;

use chrono::Local;

use hyprland::data::Client;
use hyprland::shared::HyprDataActiveOptional;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;

use std::time::{Duration, Instant};
use tracing::{error, info};

#[derive(Debug)]
pub struct Usage {
    pub date: String,
    pub window_name: String,
    pub usage_time_secs: u64,
}

pub struct WindowData {
    pub title: String,
    pub class: String,
    pub initial_class: String,
    pub initial_title: String,
}

// pub struct Client {
//     /// The client's [`Address`][crate::shared::Address]
//     pub address: Address,
//     /// The window location
//     pub at: (i16, i16),
//     /// The window size
//     pub size: (i16, i16),
//     /// The workspace its on
//     pub workspace: WorkspaceBasic,
//     /// Is this window floating?
//     pub floating: bool,
//     /// The internal fullscreen mode
//     pub fullscreen: FullscreenMode,
//     /// The client fullscreen mode
//     #[serde(rename = "fullscreenClient")]
//     pub fullscreen_client: FullscreenMode,
//     /// The monitor id the window is on
//     pub monitor: MonitorId,
//     /// The initial window class
//     #[serde(rename = "initialClass")]
//     pub initial_class: String,
//     /// The window class
//     pub class: String,
//     /// The initial window title
//     #[serde(rename = "initialTitle")]
//     pub initial_title: String,
//     /// The window title
//     pub title: String,
//     /// The process Id of the client
//     pub pid: i32,
//     /// Is this window running under XWayland?
//     pub xwayland: bool,
//     /// Is this window pinned?
//     pub pinned: bool,
//     /// Group members
//     pub grouped: Vec<Box<Address>>,
//     /// Is this window print on screen
//     pub mapped: bool,
//     /// The swallowed window
//     pub swallowing: Option<Box<Address>>,
//     /// When was this window last focused relatively to other windows? 0 for current, 1 previous, 2 previous before that, etc
//     #[serde(rename = "focusHistoryID")]
//     pub focus_history_id: i8,
// }
struct ThisThingWillNeverBeUsed {}
/// Updates the usage map by adding elapsed time for the given (date, window_name) key.
/// Monitors the active window and updates usage data:
/// - Uses chrono to get the current date.
/// - If a record for the current date and process already exists, increments its usage time.
/// - Otherwise, creates a new record for today.
pub fn monitor_active_window(conn: duckdb::Connection) -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true)); // creating app's state
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    let mut last_key: Option<(String, String)> = None; // (date, process name)
    let mut last_switch_time = Instant::now();

    info!("starting active window monitor loop");
    while running.load(Ordering::SeqCst) {
        // loop start
        let current_date = Local::now().format("%Y-%m-%d").to_string();
        //TODO: after we get active window and process it we will need to divide it and
        //refactor extract_process_name or process_complex_names to return both
        //window name and window sub name
        if let Ok(Some(active_window)) = Client::get_active() {
            // let raw_title = active_window.initial_title.clone();
            let process_name =
                process_complex_names(extract_process_name(&raw_title), &active_window);

            let current_key = (current_date.clone(), process_name);

            if last_key.as_ref() != Some(&current_key) {
                if let Some(ref prev_key) = last_key {
                    let elapsed = last_switch_time.elapsed();

                    log_activity(&conn, &prev_key.1, elapsed.as_secs())
                        .expect("failed to log activity");
                }
                last_key = Some(current_key.clone());
                last_switch_time = Instant::now();

                info!("switched to: {}", current_key.1);
            }
        }
        sleep(Duration::from_millis(500));
    }
    // save last record before shutting down
    if let Some(ref final_key) = last_key {
        let elapsed_final = last_switch_time.elapsed();
        if let Err(e) = log_activity(&conn, &final_key.1, elapsed_final.as_secs()) {
            error!("Failed to upsert final usage data on shutdown: {e}")
        }
    } else {
        info!("Final usage data successfully written.");
    };
    info!("Shutting down");
    Ok(())
}
