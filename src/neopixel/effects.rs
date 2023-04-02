use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::strip::color::Color;

pub mod hue;
pub mod invert;
pub mod solid;
pub mod strobo;
pub mod alarm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectConfig {
    Invert(invert::InversionConfig),
    HueShift(hue::HueShiftConfig),
    SolidColor(solid::SolidColorConfig),
    Strobo(strobo::StroboConfig),
    Alarm(alarm::AlarmConfig),
}

pub trait Effect {
    type Config: Default;
    fn apply(config: &Self::Config, colors: &mut Vec<Color>, dt: Duration, rt: Option<Duration>) -> anyhow::Result<()>;
}

pub fn apply_effects(
    effects: &Vec<EffectConfig>,
    colors: &mut Vec<Color>,
    dt: Duration,
    rt: Option<Duration>,
) -> anyhow::Result<()> {
    for effect in effects {
        match effect {
            EffectConfig::HueShift(config) => hue::HueShiftEffect::apply(config, colors, dt, rt)?,
            EffectConfig::SolidColor(config) => solid::SolidColorEffect::apply(config, colors, dt, rt)?,
            EffectConfig::Strobo(config) => strobo::StroboEffect::apply(config, colors, dt, rt)?,
            EffectConfig::Invert(config) => invert::InversionEffect::apply(config, colors, dt, rt)?,
            EffectConfig::Alarm(config) => alarm::AlarmEffect::apply(config, colors, dt, rt)?,
        }
    }
    Ok(())
}
