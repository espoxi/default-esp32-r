use std::time::Duration;

use serde::{Serialize, Deserialize};

use crate::neopixel::strip::color::Color;

use super::{Effect, solid::{SolidColorEffect, SolidColorConfig}};


pub struct StroboEffect;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StroboConfig {
    pub frequency_hz: f32,
}

impl Default for StroboConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 2.0,
        }
    }
}

impl Effect for StroboEffect {
    type Config = StroboConfig;
    fn apply(config: &Self::Config,  colors: &mut Vec<Color>, t:Duration) -> anyhow::Result<()> {
        match t.as_secs_f32() % (1.0 / config.frequency_hz) {
            t if t < 0.2 && t > 0.1 => {
                SolidColorEffect::apply(&SolidColorConfig{color: Color::white()}, colors, Duration::from_secs_f32(t))?;
            }
            t if t < 0.4 => {
                SolidColorEffect::apply(&SolidColorConfig{color: Color::black()}, colors, Duration::from_secs_f32(t))?;
            }
            _ => {
                return Ok(());
            }
        }
        Ok(())
    }
}