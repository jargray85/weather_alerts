// This file re-exports the library code from main.rs for use by Tauri

// Include the main.rs content as a module
#[path = "main.rs"]
mod main;

// Re-export the public items
pub use main::WeatherApp;
pub use main::WeatherType;
pub use main::fetch_weather_data;
pub use main::determine_weather_type;

// Export the run function that can be called from Tauri
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    main::run_app()
}

