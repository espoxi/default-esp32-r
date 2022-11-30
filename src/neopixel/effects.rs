use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::strip::color::Color;

pub mod hue;
pub mod invert;
pub mod solid;
pub mod strobo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectConfig {
    Invert(invert::InversionConfig),
    HueShift(hue::HueShiftConfig),
    SolidColor(solid::SolidColorConfig),
    Strobo(strobo::StroboConfig),
}

pub trait Effect {
    type Config: Default;
    fn apply(config: &Self::Config, colors: &mut Vec<Color>, t: Duration) -> anyhow::Result<()>;
}

pub fn apply_effects(
    effects: &Vec<EffectConfig>,
    colors: &mut Vec<Color>,
    t: Duration,
) -> anyhow::Result<()> {
    for effect in effects {
        match effect {
            EffectConfig::HueShift(config) => hue::HueShiftEffect::apply(config, colors, t)?,
            EffectConfig::SolidColor(config) => solid::SolidColorEffect::apply(config, colors, t)?,
            EffectConfig::Strobo(config) => strobo::StroboEffect::apply(config, colors, t)?,
            EffectConfig::Invert(config) => invert::InversionEffect::apply(config, colors, t)?,
        }
    }
    Ok(())
}
