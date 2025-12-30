// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    // Write to a log file for debugging (check /tmp/weather_alerts.log)
    const LOG_PATH: &str = "/tmp/weather_alerts.log";
    let _ = std::fs::write(LOG_PATH, "=== Starting Weather Alerts app ===\n");

    // Don't run Tauri's event loop - it conflicts with egui
    // Tauri is only used for packaging, not for runtime functionality
    // Just run egui directly on the main thread
    let _ = std::fs::write(LOG_PATH, "Starting egui on main thread...\n");

    // Run egui on the main thread (required for windowing)
    // This will block until the window is closed
    let _ = std::fs::write(LOG_PATH, "Calling weather_alerts::run()...\n");
    match weather_alerts::run() {
        Ok(_) => {
            let _ = std::fs::write(LOG_PATH, "Egui app exited successfully\n");
        }
        Err(e) => {
            let error_msg = format!("Error running weather app: {}\n", e);
            let _ = std::fs::write(LOG_PATH, &error_msg);
            eprintln!("{}", error_msg);
            // Show error dialog on macOS
            #[cfg(target_os = "macos")]
            {
                use std::process::Command;
                let _ = Command::new("osascript")
                    .arg("-e")
                    .arg(format!("display dialog \"Weather app error: {}\" buttons {{\"OK\"}} default button \"OK\"", error_msg))
                    .output();
            }
        }
    }

    let _ = std::fs::write(LOG_PATH, "Main function exiting\n");
}
