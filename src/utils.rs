use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use tracing::info;

pub fn get_app_data_dir() -> Result<PathBuf, io::Error> {
    let home_dir = env::var("HOME").map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("HOME environment variable not found: {}", e),
        )
    })?;

    let mut data_dir = PathBuf::from(home_dir);
    data_dir.push(".local");
    data_dir.push("share");
    data_dir.push("ctvtcntr");

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|e| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Failed to create data directory at {:?}: {}", data_dir, e),
            )
        })?;
        info!("Created data directory at: {:?}", data_dir);
    }

    Ok(data_dir)
}
