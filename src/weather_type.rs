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

