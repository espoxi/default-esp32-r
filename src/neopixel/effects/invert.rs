use std::time::Duration;

use serde::{Serialize, Deserialize};

use crate::neopixel::strip::color::Color;

use super::Effect;


pub struct InversionEffect;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InversionConfig {

}

impl Default for InversionConfig {
    fn default() -> Self {
        Self {
            
        }
    }
}

#[allow(unused_variables)]
impl Effect for InversionEffect {
    type Config = InversionConfig;
    fn apply(config: &Self::Config,  colors: &mut Vec<Color>, _:Duration) -> anyhow::Result<()> {
        let size = colors.len();
        for i in 0..size {
            (colors[i as usize]) = Color::white()-colors[i as usize];
        }
        Ok(())
    }
}