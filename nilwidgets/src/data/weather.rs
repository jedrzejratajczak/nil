use serde::Deserialize;

// User: change these to your location
const LATITUDE: f64 = 51.10;
const LONGITUDE: f64 = 17.04;

pub struct Weather {
    pub temperature: f32,
    pub weather_code: u8,
}

#[derive(Deserialize)]
struct ApiResponse {
    current: CurrentWeather,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    weather_code: u8,
}

pub fn fetch_weather() -> Result<Weather, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code&timezone=auto",
        LATITUDE, LONGITUDE
    );

    let body: String = ureq::get(&url).call()?.body_mut().read_to_string()?;
    let resp: ApiResponse = serde_json::from_str(&body)?;

    Ok(Weather {
        temperature: resp.current.temperature_2m as f32,
        weather_code: resp.current.weather_code,
    })
}

/// Map WMO weather code to Unicode emoji.
pub fn weather_icon(code: u8) -> &'static str {
    match code {
        0 => "☀️",             // Clear sky
        1 => "🌤️",            // Mainly clear
        2 => "⛅",              // Partly cloudy
        3 => "☁️",             // Overcast
        45 | 48 => "🌫️",      // Fog
        51 | 53 | 55 => "🌦️", // Drizzle
        56 | 57 => "🌧️",      // Freezing drizzle
        61 | 63 | 65 => "🌧️", // Rain
        66 | 67 => "🌨️",      // Freezing rain
        71 | 73 | 75 => "❄️",  // Snow
        77 => "🌨️",            // Snow grains
        80 | 81 | 82 => "🌧️", // Rain showers
        85 | 86 => "🌨️",      // Snow showers
        95 => "⛈️",            // Thunderstorm
        96 | 99 => "⛈️",      // Thunderstorm with hail
        _ => "❓",
    }
}
