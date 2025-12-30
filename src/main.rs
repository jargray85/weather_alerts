use std::env;
use serde::Deserialize;
use reqwest::Client;
use eframe::{egui, App, Frame};
use dotenv::dotenv;
use serde_json;

pub struct WeatherApp {
    weather_data: Option<String>,
    daily_weather_description: Option<String>,
    location: Option<String>,
    animation_time: f64,
    weather_type: WeatherType,
}

#[derive(Clone, Copy)]
pub enum WeatherType {
    Clear,
    PartlyCloudy,
    Cloudy,
    Rain,
    Snow,
    Thunderstorm,
    Fog,
}

impl App for WeatherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Update animation time
        self.animation_time += ctx.input(|i| i.unstable_dt) as f64;
        
        // Request continuous repaint to keep animation running
        ctx.request_repaint();

        // Determine background color based on weather data
        let weather_info = self.weather_data.clone().unwrap_or_default().to_lowercase();
        let background_color = if weather_info.contains("current weather: clear sky") || weather_info.contains("current weather: partly cloudy sky") {
            egui::Color32::from_rgb(135, 206, 250)  // Blue for sunny/partly sunny
        } else if weather_info.contains("current weather: cloudy") || weather_info.contains("current weather: overcast") {
            egui::Color32::GRAY                     // Gray for cloudy/overcast
        } else if weather_info.contains("current weather: rain") || weather_info.contains("current weather: snow") {
            egui::Color32::DARK_GRAY                // Dark Gray for stormy weather
        } else {
            egui::Color32::WHITE                    // Default color
        };

        // Apply background color
        let _frame = egui::Frame::default().fill(background_color);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let heading_text = if let (Some(ref location), Some(ref desc)) = (&self.location, &self.daily_weather_description) {
                    format!("Today's weather for {} - {}", location, desc)
                } else {
                    "Today's Weather".to_string()
                };

                ui.label(egui::RichText::new(heading_text).size(32.0).strong().color(egui::Color32::WHITE));
                ui.separator();
                ui.add_space(20.0); // Increased padding with a margin of 20.0

                if let Some(ref data) = self.weather_data {
                    ui.label(egui::RichText::new(data).size(16.0).color(egui::Color32::WHITE)); // Consistent font sizing for the weather data
                } else {
                    ui.spinner();
                    ui.label(egui::RichText::new("Fetching weather data...").size(16.0).color(egui::Color32::WHITE));
                }

                ui.add_space(30.0);

                // Display weather animation
                ui.add_space(10.0);

                // Draw weather animation based on weather type
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(300.0, 300.0),
                    egui::Sense::hover()
                );

                // Draw the animation
                self.draw_weather_animation(ui.painter(), rect, self.animation_time);
            });
        });
    }
}

pub async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Fetch weather data
    let (weather_data, daily_weather_description, city) = fetch_weather_data().await?;

    // Determine weather type from description
    let weather_type = determine_weather_type(&daily_weather_description);

    // Create the app instance
    let app = WeatherApp {
        weather_data: Some(weather_data),
        daily_weather_description: Some(daily_weather_description),
        location: Some(city),
        animation_time: 0.0,
        weather_type,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_app().await
}

pub async fn fetch_weather_data() -> Result<(String, String, String), Box<dyn std::error::Error>> {
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

pub fn determine_weather_type(description: &str) -> WeatherType {
    let desc_lower = description.to_lowercase();
    if desc_lower.contains("snow") {
        WeatherType::Snow
    } else if desc_lower.contains("rain") || desc_lower.contains("drizzle") {
        WeatherType::Rain
    } else if desc_lower.contains("thunder") || desc_lower.contains("storm") {
        WeatherType::Thunderstorm
    } else if desc_lower.contains("fog") || desc_lower.contains("mist") {
        WeatherType::Fog
    } else if desc_lower.contains("cloudy") || desc_lower.contains("overcast") {
        WeatherType::Cloudy
    } else if desc_lower.contains("partly") || desc_lower.contains("few clouds") || desc_lower.contains("scattered") {
        WeatherType::PartlyCloudy
    } else {
        WeatherType::Clear
    }
}

impl WeatherApp {
    fn draw_weather_animation(&self, painter: &egui::Painter, rect: egui::Rect, time: f64) {
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.4;
        
        match self.weather_type {
            WeatherType::Clear => {
                // Animated sun
                let sun_radius = radius * 0.6;
                let rays = 8;
                for i in 0..rays {
                    let angle = (i as f64 / rays as f64) * std::f64::consts::TAU + time * 0.5;
                    let cos_a = angle.cos() as f32;
                    let sin_a = angle.sin() as f32;
                    let start = center + egui::Vec2::new(cos_a, sin_a) * sun_radius;
                    let end = center + egui::Vec2::new(cos_a, sin_a) * (sun_radius * 1.3);
                    painter.line_segment([start, end], egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 255, 0)));
                }
                painter.circle_filled(center, sun_radius, egui::Color32::from_rgb(255, 255, 0));
            }
            WeatherType::PartlyCloudy => {
                // Partly cloudy: sun with clouds
                let sun_radius = radius * 0.3;
                painter.circle_filled(center + egui::Vec2::new(-radius * 0.3, -radius * 0.3), sun_radius, egui::Color32::from_rgb(255, 255, 200));
                
                // Animated clouds
                for i in 0..3 {
                    let offset_x = (i as f32 - 1.0) * radius * 0.4 + (time * 20.0).sin() as f32 * 10.0;
                    let offset_y = radius * 0.2 + (time * 15.0).cos() as f32 * 5.0;
                    let cloud_pos = center + egui::Vec2::new(offset_x, offset_y);
                    self.draw_cloud(painter, cloud_pos, radius * 0.3);
                }
            }
            WeatherType::Cloudy => {
                // Animated clouds
                for i in 0..4 {
                    let offset_x = (i as f32 - 1.5) * radius * 0.5 + (time * 15.0 + i as f64).sin() as f32 * 15.0;
                    let offset_y = (time * 12.0 + i as f64 * 0.5).cos() as f32 * 10.0;
                    let cloud_pos = center + egui::Vec2::new(offset_x, offset_y);
                    self.draw_cloud(painter, cloud_pos, radius * 0.35);
                }
            }
            WeatherType::Rain => {
                // Animated rain drops
                let drop_count = 30;
                for i in 0..drop_count {
                    let x = center.x + ((i % 10) as f32 - 5.0) * radius * 0.15;
                    let cycle_time = time + i as f64 * 0.1;
                    let y = center.y - radius + ((cycle_time * 200.0) as f32 % (radius * 2.0));
                    let drop_pos = egui::Pos2::new(x, y);
                    painter.line_segment(
                        [drop_pos, drop_pos + egui::Vec2::new(0.0, radius * 0.15)],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255))
                    );
                }
                // Cloud above
                self.draw_cloud(painter, center + egui::Vec2::new(0.0, -radius * 0.5), radius * 0.4);
            }
            WeatherType::Snow => {
                // Animated snowflakes - larger, slower, more detailed
                let flake_count = 15;
                for i in 0..flake_count {
                    // Slower vertical movement than rain
                    let base_x = center.x + ((i % 6) as f32 - 2.5) * radius * 0.25;
                    let cycle_time = time + i as f64 * 0.2;
                    // Slower fall speed (80 instead of 150-200)
                    let y = center.y - radius + ((cycle_time * 80.0) as f32 % (radius * 2.0));
                    // Horizontal drift/wind effect
                    let drift = (cycle_time * 0.5 + i as f64 * 0.3).sin() as f32 * radius * 0.15;
                    let x = base_x + drift;
                    let flake_pos = egui::Pos2::new(x, y);
                    
                    // Larger, more detailed snowflake
                    let size = radius * 0.12;
                    let stroke_width = 2.5;
                    
                    // Draw 6-pointed snowflake (more realistic)
                    // Main arms (6 lines crossing at center - 60 degrees apart)
                    for arm in 0..6 {
                        let angle = (arm as f32 * std::f32::consts::PI / 3.0) + (cycle_time * 0.3) as f32;
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();
                        let end = flake_pos + egui::Vec2::new(cos_a, sin_a) * size;
                        painter.line_segment(
                            [flake_pos, end],
                            egui::Stroke::new(stroke_width, egui::Color32::WHITE)
                        );
                        // Add small branches on each arm
                        let branch_size = size * 0.4;
                        let branch_angle1 = angle + std::f32::consts::PI / 6.0;
                        let branch_angle2 = angle - std::f32::consts::PI / 6.0;
                        let branch_start = flake_pos + egui::Vec2::new(cos_a, sin_a) * (size * 0.6);
                        let branch_end1 = branch_start + egui::Vec2::new(branch_angle1.cos(), branch_angle1.sin()) * branch_size;
                        let branch_end2 = branch_start + egui::Vec2::new(branch_angle2.cos(), branch_angle2.sin()) * branch_size;
                        painter.line_segment(
                            [branch_start, branch_end1],
                            egui::Stroke::new(stroke_width * 0.7, egui::Color32::WHITE)
                        );
                        painter.line_segment(
                            [branch_start, branch_end2],
                            egui::Stroke::new(stroke_width * 0.7, egui::Color32::WHITE)
                        );
                    }
                    
                    // Add a soft glow effect (small circle) - use lighter white
                    painter.circle_filled(flake_pos, size * 0.3, egui::Color32::from_rgb(240, 240, 255));
                }
                // Cloud above
                self.draw_cloud(painter, center + egui::Vec2::new(0.0, -radius * 0.5), radius * 0.4);
            }
            WeatherType::Thunderstorm => {
                // Lightning with rain
                let drop_count = 25;
                for i in 0..drop_count {
                    let x = center.x + ((i % 10) as f32 - 5.0) * radius * 0.15;
                    let cycle_time = time + i as f64 * 0.1;
                    let y = center.y - radius + ((cycle_time * 200.0) as f32 % (radius * 2.0));
                    let drop_pos = egui::Pos2::new(x, y);
                    painter.line_segment(
                        [drop_pos, drop_pos + egui::Vec2::new(0.0, radius * 0.15)],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 80, 120))
                    );
                }
                // Lightning bolt
                let lightning_time = (time * 3.0) as i32;
                if lightning_time % 2 == 0 {
                    let bolt_start = center + egui::Vec2::new(-radius * 0.2, -radius * 0.3);
                    let bolt_mid = center + egui::Vec2::new(0.0, 0.0);
                    let bolt_end = center + egui::Vec2::new(radius * 0.2, radius * 0.4);
                    painter.line_segment([bolt_start, bolt_mid], egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 255, 200)));
                    painter.line_segment([bolt_mid, bolt_end], egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 255, 200)));
                }
                // Dark cloud
                self.draw_cloud(painter, center + egui::Vec2::new(0.0, -radius * 0.5), radius * 0.4);
            }
            WeatherType::Fog => {
                // Animated fog/mist
                for i in 0..5 {
                    let offset_x = (i as f32 - 2.0) * radius * 0.3 + (time * 10.0 + i as f64).sin() as f32 * radius * 0.2;
                    let offset_y = (time * 8.0 + i as f64 * 0.3).cos() as f32 * radius * 0.1;
                    let fog_pos = center + egui::Vec2::new(offset_x, offset_y);
                    painter.circle_filled(fog_pos, radius * 0.25, egui::Color32::from_rgb(200, 200, 200));
                }
            }
        }
    }
    
    fn draw_cloud(&self, painter: &egui::Painter, center: egui::Pos2, size: f32) {
        let color = egui::Color32::from_rgb(200, 200, 200);
        // Draw cloud as overlapping circles
        painter.circle_filled(center, size, color);
        painter.circle_filled(center + egui::Vec2::new(-size * 0.6, 0.0), size * 0.8, color);
        painter.circle_filled(center + egui::Vec2::new(size * 0.6, 0.0), size * 0.8, color);
        painter.circle_filled(center + egui::Vec2::new(0.0, size * 0.4), size * 0.7, color);
    }
}
