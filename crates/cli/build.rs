//! Build script for scan3data CLI
//!
//! Generates build-time metadata for version output

use std::env;

fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");

    // Set build-time environment variables for use in clap
    println!(
        "cargo:rustc-env=BUILT_HOST={}",
        env::var("HOST").unwrap_or_else(|_| "unknown".to_string())
    );
    println!(
        "cargo:rustc-env=BUILT_GIT_COMMIT_HASH={}",
        env::var("GIT_COMMIT_HASH").unwrap_or_else(|_| "unknown".to_string())
    );
    println!(
        "cargo:rustc-env=BUILT_TIME_UTC={}",
        chrono::Utc::now().to_rfc3339()
    );
}
