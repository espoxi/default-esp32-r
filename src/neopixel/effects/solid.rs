use std::{ops::Range, time::Duration};

use serde::{Deserialize, Serialize};

use crate::neopixel::strip::color::FColor;

use super::Effect;

pub struct SolidColorEffect;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolidColorConfig {
    pub color: FColor,
    pub range: Range<u16>,
}

impl Default for SolidColorConfig {
    fn default() -> Self {
        Self {
            color: FColor::black(),
            range: 0..30,
        }
    }
}

impl Effect for SolidColorEffect {
    type Config = SolidColorConfig;
    fn apply(config: &Self::Config, colors: &mut Vec<FColor>, _: Duration, _ : Option<Duration>) -> anyhow::Result<()> {
        for i in config.range.clone() {
            (colors[i as usize]) = config.color;
        }
        Ok(())
    }
}
