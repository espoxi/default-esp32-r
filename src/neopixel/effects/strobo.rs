use std::{ops::Range, time::Duration};

use serde::{Deserialize, Serialize};

use crate::neopixel::strip::color::default::Color;

use super::{
    solid::{SolidColorConfig, SolidColorEffect},
    Effect,
};

pub struct StroboEffect;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StroboConfig {
    pub frequency_hz: f32,
    pub range: Range<u16>,
}

impl Default for StroboConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 2.0,
            range: 0..30,
        }
    }
}

impl Effect for StroboEffect {
    type Config = StroboConfig;
    fn apply(config: &Self::Config, colors: &mut Vec<Color>, dt: Duration, rt : Option<Duration>) -> anyhow::Result<()> {
        let t = dt;
        match t.as_secs_f32() % (1.0 / config.frequency_hz) {
            t if t < 0.2 && t > 0.1 => {
                SolidColorEffect::apply(
                    &SolidColorConfig {
                        color: Color::white(),
                        range: config.range.clone(),
                    },
                    colors,
                    Duration::from_secs_f32(t),
                    rt,
                )?;
            }
            t if t < 0.4 => {
                SolidColorEffect::apply(
                    &SolidColorConfig {
                        color: Color::black(),
                        range: config.range.clone(),
                    },
                    colors,
                    Duration::from_secs_f32(t),
                    rt,
                )?;
            }
            _ => {
                return Ok(());
            }
        }
        Ok(())
    }
}
