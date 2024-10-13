use std::env;
use serde::Deserialize;
use reqwest::Client;
use eframe::{egui, App, Frame};
use dotenv::dotenv;

struct WeatherApp {
    weather_data: Option<String>,
    daily_weather_description: Option<String>,
}

impl App for WeatherApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let heading_text = if let Some(ref desc) = self.daily_weather_description {
                format!("Today's Weather - {}", desc)
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
    let (weather_data, daily_weather_description) = fetch_weather_data().await?;

    // Create the app instance
    let app = WeatherApp {
        weather_data: Some(weather_data),
        daily_weather_description: Some(daily_weather_description),
    };

    // Run the GUI application
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Weather Alerts",         // Application title
        native_options,           // Native options
        Box::new(|_cc| Box::new(app)), // App creator closure
    );

    Ok(())
}

async fn fetch_weather_data() -> Result<(String, String), Box<dyn std::error::Error>> {
    // Load environment variables
    let api_key = env::var("OPENWEATHERMAP_API_KEY")?;
    let city = env::var("CITY").unwrap_or_else(|_| "Valparaiso".to_string());
    let country_code = env::var("COUNTRY_CODE").unwrap_or_else(|_| "US".to_string());

    let client = Client::new();

    // Get coordinates
    let (lat, lon) = get_coordinates(&client, &city, &country_code, &api_key).await?;

    // Get weather data
    let weather_data = get_weather_data(&client, lat, lon, &api_key).await?;

    // Format weather data and get daily_weather_description
    let (weather_string, daily_weather_description) = format_weather_data(&weather_data);

    Ok((weather_string, daily_weather_description))
}

#[derive(Debug, Deserialize)]
struct GeoResponse {
    name: String,
    lat: f64,
    lon: f64,
    country: String,
}

#[derive(Debug, Deserialize)]
struct Weather {
    id: u16,
    main: String,
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
    day: f64,
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
    let daily_weather_description = today.weather[0].description.clone();

    let today_summary = &today.summary;

    let chance_of_rain_tomorrow = if let Some(tomorrow) = tomorrow {
        (tomorrow.pop.min(1.0) * 100.0).round()
    } else {
        0.0
    };

    let formatted_data = format!(
        "Weather: {}\n\
        Temperature: {:.1}°F (Feels like {:.1}°F)\n\
        Humidity: {}%\n\
        Wind: {:.1} mph {}\n\
        Chance of Rain Today: {:.0}%\n\
        Chance of Rain Tomorrow: {:.0}%\n\
        Summary: {}",
        weather_description,
        temp,
        feels_like,
        humidity,
        wind_speed,
        wind_direction,
        chance_of_rain_today,
        chance_of_rain_tomorrow,
        today_summary
    );

    (formatted_data, daily_weather_description)
}

fn degrees_to_cardinal(degrees: u16) -> &'static str {
    let dirs = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
        "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW",
    ];
    let index = (((degrees as f32 + 11.25) / 22.5) as usize) % 16;
    dirs[index]
}

