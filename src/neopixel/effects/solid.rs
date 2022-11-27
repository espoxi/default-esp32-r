use std::time::Duration;

use crate::neopixel::strip::color::Color;

use super::Effect;


pub struct SolidColorEffect {
    pub config: SolidColorConfig,
}

pub struct SolidColorConfig {
    pub color: Color,
}

impl Default for SolidColorConfig {
    fn default() -> Self {
        Self {
            color: Color::black(),
        }
    }
}

impl Effect for SolidColorEffect {
    type Config = SolidColorConfig;
    fn new(config: Self::Config) -> Self {
        Self { config }
    }
    fn apply(&self, colors: &mut Vec<Color>, t:Duration) -> anyhow::Result<()> {
        let size = colors.len();
        for i in 0..size {
            (colors[i as usize]) = self.config.color;
        }
        Ok(())
    }
}