use std::sync::{Arc, Mutex};
use eframe::{egui, App, Frame};
use crate::weather_type::{WeatherType, determine_weather_type};
use crate::weather::fetch_weather_data;

pub struct WeatherApp {
    weather_data: Option<String>,
    daily_weather_description: Option<String>,
    location: Option<String>,
    animation_time: f64,
    weather_type: WeatherType,
    weather_fetch_in_progress: Arc<Mutex<bool>>,
    weather_result: Arc<Mutex<Option<(String, Option<String>, Option<String>, WeatherType)>>>,
}

impl App for WeatherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Check if weather data fetch is in progress and start it if needed
        {
            let mut in_progress = self.weather_fetch_in_progress.lock().unwrap();
            if self.weather_data.is_none() && !*in_progress {
                *in_progress = true;
                let result_clone = Arc::clone(&self.weather_result);
                let in_progress_clone = Arc::clone(&self.weather_fetch_in_progress);
                
                // Spawn background thread to fetch weather
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let fetch_result = rt.block_on(fetch_weather_data());
                    let result = match fetch_result {
                        Ok((data, desc, city)) => {
                            let wt = determine_weather_type(&desc);
                            Some((data, Some(desc), Some(city), wt))
                        }
                        Err(e) => {
                            let error_msg = if e.to_string().contains("environment variable not found") {
                                "API Key Missing: Please set OPENWEATHERMAP_API_KEY environment variable.\n\nFor packaged apps, you can:\n1. Create a .env file in the app's directory\n2. Or set it as a system environment variable\n3. Or run: export OPENWEATHERMAP_API_KEY='your-key' before launching"
                            } else {
                                &format!("Error fetching weather data: {}", e)
                            };
                            Some((error_msg.to_string(), None, None, WeatherType::Clear))
                        }
                    };
                    *result_clone.lock().unwrap() = result;
                    *in_progress_clone.lock().unwrap() = false;
                });
            }
        }
        
        // Check if weather data is ready
        if let Ok(mut result) = self.weather_result.lock() {
            if let Some((data, desc, city, wt)) = result.take() {
                self.weather_data = Some(data);
                self.daily_weather_description = desc;
                self.location = city;
                self.weather_type = wt;
            }
        }
        
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

impl WeatherApp {
    pub fn new() -> Self {
        Self {
            weather_data: None,
            daily_weather_description: None,
            location: None,
            animation_time: 0.0,
            weather_type: WeatherType::Clear,
            weather_fetch_in_progress: Arc::new(Mutex::new(false)),
            weather_result: Arc::new(Mutex::new(None)),
        }
    }

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
                // Layered static clouds (no motion)
                // Back layer - larger, darker clouds
                self.draw_cloud(painter, center + egui::Vec2::new(-radius * 0.3, -radius * 0.2), radius * 0.4);
                self.draw_cloud(painter, center + egui::Vec2::new(radius * 0.3, -radius * 0.15), radius * 0.38);
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
        // Use slightly different shades for depth
        let base_color = egui::Color32::from_rgb(200, 200, 200);
        let darker_color = egui::Color32::from_rgb(180, 180, 180);
        
        // Draw cloud as overlapping circles for a fluffy appearance
        painter.circle_filled(center, size, base_color);
        painter.circle_filled(center + egui::Vec2::new(-size * 0.6, 0.0), size * 0.8, base_color);
        painter.circle_filled(center + egui::Vec2::new(size * 0.6, 0.0), size * 0.8, base_color);
        painter.circle_filled(center + egui::Vec2::new(0.0, size * 0.4), size * 0.7, darker_color);
        painter.circle_filled(center + egui::Vec2::new(-size * 0.4, size * 0.3), size * 0.6, base_color);
        painter.circle_filled(center + egui::Vec2::new(size * 0.4, size * 0.3), size * 0.6, base_color);
    }
}

