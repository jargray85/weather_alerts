use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct WeatherRequest {
    city: String,
    country_code: String,
}

#[derive(Serialize)]
struct WeatherResponse {
    weather_data: serde_json::Value,
    daily_weather_description: String,
    city: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv::dotenv().ok();
    
    let api_key = env::var("OPENWEATHERMAP_API_KEY")
        .expect("OPENWEATHERMAP_API_KEY must be set in .env file");

    // Build the application router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/weather", post(handle_weather_request))
        .layer(CorsLayer::permissive()) // Allow all origins for now
        .with_state(api_key);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");
    
    println!("Weather proxy server running on http://0.0.0.0:3000");
    println!("API key loaded successfully");
    
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

async fn health_check() -> &'static str {
    "Weather Proxy Server is running! Use POST /api/weather to get weather data."
}

async fn handle_weather_request(
    axum::extract::State(api_key): axum::extract::State<String>,
    Json(request): Json<WeatherRequest>,
) -> Result<ResponseJson<WeatherResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let client = reqwest::Client::new();

    // Get coordinates
    let geo_url = format!(
        "http://api.openweathermap.org/geo/1.0/direct?q={},{}&limit=1&appid={}",
        request.city, request.country_code, api_key
    );

    let geo_res = client
        .get(&geo_url)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    error: format!("Failed to get coordinates: {}", e),
                }),
            )
        })?;

    let geo_data: Vec<serde_json::Value> = geo_res
        .json()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    error: format!("Failed to parse coordinates: {}", e),
                }),
            )
        })?;

    let (lat, lon) = if let Some(location) = geo_data.first() {
        (
            location["lat"].as_f64().unwrap_or(0.0),
            location["lon"].as_f64().unwrap_or(0.0),
        )
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse {
                error: "Unable to get location coordinates".to_string(),
            }),
        ));
    };

    // Get weather data
    let weather_url = format!(
        "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&units=imperial&exclude=minutely,hourly,alerts&appid={}",
        lat, lon, api_key
    );

    let weather_res = client
        .get(&weather_url)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    error: format!("Failed to get weather data: {}", e),
                }),
            )
        })?;

    let weather_data: serde_json::Value = weather_res
        .json()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    error: format!("Failed to parse weather data: {}", e),
                }),
            )
        })?;

    // Extract daily weather description
    let daily_weather_description = weather_data["daily"][0]["weather"][0]["description"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    Ok(ResponseJson(WeatherResponse {
        weather_data,
        daily_weather_description,
        city: request.city,
    }))
}

