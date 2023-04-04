use std::{ops::Range, time::Duration};

use serde::{Deserialize, Serialize};

use crate::neopixel::strip::color::default::Color;

use super::Effect;

pub struct InversionEffect;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InversionConfig {
    pub range: Range<u16>,
}

impl Default for InversionConfig {
    fn default() -> Self {
        Self { range: 0..30 }
    }
}

#[allow(unused_variables)]
impl Effect for InversionEffect {
    type Config = InversionConfig;
    fn apply(config: &Self::Config, colors: &mut Vec<Color>, _: Duration, _:Option<Duration>) -> anyhow::Result<()> {
        for i in config.range.clone() {
            (colors[i as usize]) = Color::white() - colors[i as usize];
        }
        Ok(())
    }
}
