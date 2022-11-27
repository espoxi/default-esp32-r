use std::time::Duration;

use crate::neopixel::strip::color::Color;

use super::Effect;


pub struct HueShiftEffect {
    pub config: HueShiftConfig,
}

pub struct HueShiftConfig {
    pub degrees_per_second: f32,
    pub degrees_per_led: f32,
}

impl Default for HueShiftConfig {
    fn default() -> Self {
        Self {
            degrees_per_second: 5.0,
            degrees_per_led: 0.0,
        }
    }
}

impl Effect for HueShiftEffect {
    type Config = HueShiftConfig;
    fn new(config: Self::Config) -> Self {
        Self { config }
    }
    fn apply(&self, colors: &mut Vec<Color>, t:Duration) -> anyhow::Result<()> {
        let size = colors.len();
        for i in 0..size {
            (colors[i as usize]).shift_hue_deg(self.config.degrees_per_second * t.as_secs_f32() + self.config.degrees_per_led * i as f32);
        }
        Ok(())
    }
}

