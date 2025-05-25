use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let home_dir = env::var("HOME")
        .expect("Failed to get HOME directory. This script is intended for Linux-like systems.");
    let mut data_dir = PathBuf::from(home_dir);
    data_dir.push(".local");
    data_dir.push("share");
    data_dir.push("ctvtcntr");

    // Create the directory if it doesn't exist
    if !data_dir.exists() {
        println!("cargo:info=Creating data directory at: {:?}", data_dir);
        fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    }
    println!("cargo:rustc-env=CTVTCNTR_DATA_DIR={}", data_dir.display());
}
