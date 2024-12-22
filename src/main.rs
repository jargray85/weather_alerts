use std::env;
use serde::Deserialize;
use reqwest::Client;
use eframe::{egui, App, Frame};
use dotenv::dotenv;
use serde_json;

struct WeatherApp {
    weather_data: Option<String>,
    daily_weather_description: Option<String>,
    location: Option<String>
}

impl App for WeatherApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        let _ = frame;
        egui::CentralPanel::default().show(ctx, |ui| {
            let heading_text = if let (Some(ref location), Some(ref desc)) = (&self.location, &self.daily_weather_description) {
                format!("Today's weather for {} - {}", location, desc)
            } else {
                "Today's Weather".to_string()
            };
            ui.heading(heading_text);
            if let Some(ref data) = self.weather_data {
                ui.separator();
                ui.label(data);
            } else {
                ui.spinner();
                ui.label("Fetching weather data...");
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Fetch weather data
    let (weather_data, daily_weather_description, city) = fetch_weather_data().await?;

    // Create the app instance
    let app = WeatherApp {
        weather_data: Some(weather_data),
        daily_weather_description: Some(daily_weather_description),
        location: Some(city),
    };

    // Run the GUI application
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Weather Alerts",         // Application title
        native_options,           // Native options
        Box::new(|_cc| Box::new(app)), // App creator closure
    );

    Ok(())
}

async fn fetch_weather_data() -> Result<(String, String, String), Box<dyn std::error::Error>> {
    // Load environment variables (no longer needed for city and country)
    let api_key = env::var("OPENWEATHERMAP_API_KEY")?;

    // Get user's location
    let (city, country_code) = get_user_location().await?;

    let client = Client::new();

    // Get coordinates
    let (lat, lon) = get_coordinates(&client, &city, &country_code, &api_key).await?;

    // Get weather data
    let weather_data = get_weather_data(&client, lat, lon, &api_key).await?;

    // Format weather data and get daily_weather_description
    let (weather_string, daily_weather_description) = format_weather_data(&weather_data);

    Ok((weather_string, daily_weather_description, city))
}

async fn get_user_location() -> Result<(String, String), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Set a reasonable timeout
    let res = client.get("http://ip-api.com/json/")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    if res.status().is_success() {
        let json: serde_json::Value = res.json().await?;
        let city = json["city"].as_str().unwrap_or("Unknown City").to_string();
        let country_code = json["countryCode"].as_str().unwrap_or("US").to_string();

        Ok((city, country_code))
    } else {
        Err("Failed to get user location".into())
    }
}

#[derive(Debug, Deserialize)]
struct GeoResponse {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Deserialize)]
struct Weather {
    description: String,
}

#[derive(Debug, Deserialize)]
struct Current {
    temp: f64,
    feels_like: f64,
    humidity: u8,
    wind_speed: f64,
    wind_deg: u16,
    weather: Vec<Weather>,
}

#[derive(Debug, Deserialize)]
struct Daily {
    #[serde(default)]
    pop: f64,
    #[serde(default)]
    summary: String,
    temp: DailyTemp,
    weather: Vec<Weather>,
}

#[derive(Debug, Deserialize)]
struct DailyTemp {
    min: f64,
    max: f64,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    current: Current,
    daily: Vec<Daily>,
}

async fn get_coordinates(
    client: &Client,
    city: &str,
    country_code: &str,
    api_key: &str,
) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let geo_url = format!(
        "http://api.openweathermap.org/geo/1.0/direct?q={},{}&limit=1&appid={}",
        city, country_code, api_key
    );

    let res = client.get(&geo_url).send().await?;
    let geo_data: Vec<GeoResponse> = res.json().await?;

    if let Some(location) = geo_data.first() {
        Ok((location.lat, location.lon))
    } else {
        Err("Unable to get location coordinates.".into())
    }
}

async fn get_weather_data(
    client: &Client,
    lat: f64,
    lon: f64,
    api_key: &str,
) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
    let weather_url = format!(
        "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&units=imperial&exclude=minutely,hourly,alerts&appid={}",
        lat, lon, api_key
    );

    let res = client.get(&weather_url).send().await?;
    let text = res.text().await?;

    let weather_data: WeatherResponse = serde_json::from_str(&text)?;
    Ok(weather_data)
}

fn format_weather_data(weather_data: &WeatherResponse) -> (String, String) {
    let current = &weather_data.current;
    let today = &weather_data.daily[0];
    let tomorrow = weather_data.daily.get(1);

    let weather_description = &current.weather[0].description;
    let temp = current.temp;
    let feels_like = current.feels_like;
    let humidity = current.humidity;
    let wind_speed = current.wind_speed;
    let wind_deg = current.wind_deg;

    let wind_direction = degrees_to_cardinal(wind_deg);

    // Ensure pop is within 0.0 to 1.0
    let chance_of_rain_today = (today.pop.min(1.0) * 100.0).round();
    let daily_weather_description = capitalize_first_letter(&today.weather[0].description);

    let today_summary = &today.summary;

    let chance_of_rain_tomorrow = if let Some(tomorrow) = tomorrow {
        (tomorrow.pop.min(1.0) * 100.0).round()
    } else {
        0.0
    };

    let temp_min = today.temp.min;
    let temp_max = today.temp.max;

    let formatted_data = format!(
        r"Summary: {}
        Current weather: {}
        Temperature: {:.1}°F (Feels like {:.1}°F)
        High: {:.1}°F
        Low: {:.1}°F
        Humidity: {}%
        Wind: {:.1} mph {}
        Chance of Rain Today: {:.0}%
        Chance of Rain Tomorrow: {:.0}% ",
        today_summary,
        weather_description,
        temp,
        feels_like,
        temp_max,
        temp_min,
        humidity,
        wind_speed,
        wind_direction,
        chance_of_rain_today,
        chance_of_rain_tomorrow,
    );

    (formatted_data, daily_weather_description)
}

fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn degrees_to_cardinal(degrees: u16) -> &'static str {
    let dirs = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
        "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW",
    ];
    let index = (((degrees as f32 + 11.25) / 22.5) as usize) % 16;
    dirs[index]
}

