use std::env;
use reqwest::Client;
use serde::Deserialize;
use serde_json;

// API response structs
#[derive(Debug, Deserialize)]
pub struct Weather {
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Current {
    pub temp: f64,
    pub feels_like: f64,
    pub humidity: u8,
    pub wind_speed: f64,
    pub wind_deg: u16,
    pub weather: Vec<Weather>,
}

#[derive(Debug, Deserialize)]
pub struct Daily {
    #[serde(default)]
    pub pop: f64,
    #[serde(default)]
    pub summary: String,
    pub temp: DailyTemp,
    pub weather: Vec<Weather>,
}

#[derive(Debug, Deserialize)]
pub struct DailyTemp {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize)]
pub struct WeatherResponse {
    pub current: Current,
    pub daily: Vec<Daily>,
}

// Internal proxy response struct
#[derive(Deserialize)]
struct ProxyResponse {
    weather_data: serde_json::Value,
    daily_weather_description: String,
    city: String,
}

pub async fn fetch_weather_data() -> Result<(String, String, String), Box<dyn std::error::Error>> {
    // Get proxy server URL (defaults to localhost for development)
    // For production, set WEATHER_PROXY_URL environment variable or bundle it
    let proxy_url = env::var("WEATHER_PROXY_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Get user's location
    let (city, country_code) = get_user_location().await?;

    let client = Client::new();

    // Call proxy server instead of OpenWeatherMap directly
    let request_body = serde_json::json!({
        "city": city,
        "country_code": country_code
    });

    let response = client
        .post(&format!("{}/api/weather", proxy_url))
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to weather server: {}. Make sure the proxy server is running.", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Weather server error: {}", error_text).into());
    }

    let proxy_response: ProxyResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse weather server response: {}", e))?;

    // Convert the JSON weather_data back to WeatherResponse format for formatting
    let weather_response: WeatherResponse = serde_json::from_value(proxy_response.weather_data)
        .map_err(|e| format!("Failed to parse weather data: {}", e))?;

    // Format weather data
    let (weather_string, _) = format_weather_data(&weather_response);

    Ok((
        weather_string,
        proxy_response.daily_weather_description,
        proxy_response.city,
    ))
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

pub fn format_weather_data(weather_data: &WeatherResponse) -> (String, String) {
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

    // Check if today's weather is snow
    let today_weather_desc = today.weather[0].description.to_lowercase();
    let is_snow_today = today_weather_desc.contains("snow");

    // Check if tomorrow's weather is snow
    let is_snow_tomorrow = if let Some(tomorrow) = tomorrow {
        tomorrow.weather[0].description.to_lowercase().contains("snow")
    } else {
        false
    };

    let temp_min = today.temp.min;
    let temp_max = today.temp.max;

    // Determine the precipitation type labels
    let today_precip_label = if is_snow_today { "Snow" } else { "Rain" };
    let tomorrow_precip_label = if is_snow_tomorrow { "Snow" } else { "Rain" };

    let formatted_data = format!(
        r"Summary: {}
        Current weather: {}
        Temperature: {:.1}°F (Feels like {:.1}°F)
        High: {:.1}°F
        Low: {:.1}°F
        Humidity: {}%
        Wind: {:.1} mph {}
        Chance of {} Today: {:.0}%
        Chance of {} Tomorrow: {:.0}% ",
        today_summary,
        weather_description,
        temp,
        feels_like,
        temp_max,
        temp_min,
        humidity,
        wind_speed,
        wind_direction,
        today_precip_label,
        chance_of_rain_today,
        tomorrow_precip_label,
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

