use std::env;
use dotenv::dotenv;
use serde::Deserialize;
use reqwest::Client;

#[derive(Debug, Deserialize)]
struct GeoResponse {
    lat: f64,
    lon: f64,
    // Other fields are ignored
}

#[derive(Debug, Deserialize)]
struct Weather {
    id: u16,
    main: String,
    description: String,
    // Other fields are ignored
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
struct DailyTemp {
    day: f64,
    min: f64,
    max: f64,
    // Other fields are ignored
}

#[derive(Debug, Deserialize)]
struct Daily {
    temp: DailyTemp,
    pop: f64,
    weather: Vec<Weather>,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    current: Current,
    daily: Vec<Daily>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load environment variables
    let api_key = env::var("OPENWEATHERMAP_API_KEY")?;
    println!("API Key: {}", api_key);
    let city = env::var("CITY").unwrap_or_else(|_| "Valparaiso".to_string());
    let country_code = env::var("COUNTRY_CODE").unwrap_or_else(|_| "US".to_string());

    // Create an HTTP client
    let client = Client::new();

    // Get coordinates
    let (lat, lon) = get_coordinates(&client, &city, &country_code, &api_key).await?;

    // Get weather data
    let weather_data = get_weather_data(&client, lat, lon, &api_key).await?;

    // Display weather data
    display_weather(&weather_data);

    Ok(())
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
    let weather_data: WeatherResponse = res.json().await?;
    Ok(weather_data)
}

fn display_weather(weather_data: &WeatherResponse) {
    let current = &weather_data.current;
    let today = &weather_data.daily[0];
    let tomorrow = weather_data.daily.get(1);

    let weather_description = &current.weather[0].description;
    let temp = current.temp;
    let feels_like = current.feels_like;
    let humidity = current.humidity;
    let wind_speed = current.wind_speed;
    let wind_deg = current.wind_deg;

    // Convert wind degrees to compass direction
    let wind_direction = degrees_to_cardinal(wind_deg);

    let chance_of_rain_today = today.pop * 100.0;
    let daily_weather_description = &today.weather[0].description;

    let chance_of_rain_tomorrow = if let Some(tomorrow) = tomorrow {
        tomorrow.pop * 100.0
    } else {
        0.0
    };

    println!("Weather: {}", weather_description);
    println!(
        "Temperature: {:.1}°F (Feels like {:.1}°F)",
        temp, feels_like
    );
    println!("Humidity: {}%", humidity);
    println!("Wind: {:.1} mph {}", wind_speed, wind_direction);
    println!("Chance of Rain Today: {:.0}%", chance_of_rain_today);
    println!("Chance of Rain Tomorrow: {:.0}%", chance_of_rain_tomorrow);
    println!("Today's Overview: {}", daily_weather_description);
}

fn degrees_to_cardinal(degrees: u16) -> &'static str {
    let dirs = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
        "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW",
    ];
    let index = (((degrees as f32 + 11.25) / 22.5) as usize) % 16;
    dirs[index]
}