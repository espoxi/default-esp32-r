use std::time::Duration;

use crate::neopixel::strip::color::Color;

use super::Effect;


pub struct SolidColorEffect;

#[derive(Debug)]
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
    fn apply(config: &Self::Config,  colors: &mut Vec<Color>, _:Duration) -> anyhow::Result<()> {
        let size = colors.len();
        for i in 0..size {
            (colors[i as usize]) = config.color;
        }
        Ok(())
    }
}