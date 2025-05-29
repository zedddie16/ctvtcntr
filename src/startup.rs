// Waiting for socket is basically only needed if systemd is chosen and preffered as loader
// of ctvtntr.
use std::env;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::info; // Assuming you use tracing

// --- Wait for socket function ---
pub fn wait_for_hyprland_socket(timeout_secs: u64) -> Result<(), String> {
    info!("Waiting for Hyprland socket based on XDG_RUNTIME_DIR...");
    let start_time = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        // Try to read both environment variables needed for the path
        match (
            env::var("XDG_RUNTIME_DIR"),
            env::var("HYPRLAND_INSTANCE_SIGNATURE"),
        ) {
            (Ok(xdg_runtime_dir), Ok(instance_sig)) => {
                let socket_path_str =
                    format!("{}/hypr/{}/.socket.sock", xdg_runtime_dir, instance_sig);
                let socket_path = Path::new(&socket_path_str);

                // Check if the socket file exists
                if socket_path.exists() {
                    info!("Hyprland socket found at {}", socket_path_str);
                    return Ok(());
                } else {
                    info!("Socket not found at {}, checking again...", socket_path_str);
                }
            }
            (Err(_), Ok(_)) => {
                info!("XDG_RUNTIME_DIR not found in environment, checking again...");
            }
            (Ok(_), Err(_)) => {
                info!("HYPRLAND_INSTANCE_SIGNATURE not found in environment, checking again...");
            }
            (Err(_), Err(_)) => {
                info!("XDG_RUNTIME_DIR and HYPRLAND_INSTANCE_SIGNATURE not found in environment, checking again...");
            }
        }

        // Check for timeout
        if start_time.elapsed() > timeout {
            return Err(format!(
                "Timed out after {} seconds waiting for Hyprland socket (checked XDG_RUNTIME_DIR path). Required env vars might be missing.",
                timeout_secs
            ));
        }

        // Wait before next check
        sleep(Duration::from_secs(2));
    }
}
