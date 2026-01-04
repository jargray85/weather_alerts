mod app;
mod config;
mod weather;
mod weather_type;

use app::WeatherApp;
use config::load_env_file;

pub fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    // Try to load .env file from multiple locations
    load_env_file();

    // Create the app instance - we'll fetch weather data in the update loop
    let app = WeatherApp::new();

    // Run the GUI application synchronously (not in async context)
    println!("Starting egui application...");
    eprintln!("Starting egui application...");
    
    let native_options = eframe::NativeOptions::default();
    
    let result = eframe::run_native(
        "Weather Alerts",
        native_options,
        Box::new(|_cc| {
            println!("Egui window created!");
            eprintln!("Egui window created!");
            Box::new(app)
        }),
    );
    
    if let Err(e) = result {
        let error_msg = format!("Error running egui: {}", e);
        eprintln!("{}", error_msg);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_app()
}
