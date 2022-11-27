use std::time::Duration;

use super::strip::color::Color;

pub mod hue;
pub mod solid;

pub enum EffectConfig {
    HueShift(hue::HueShiftConfig),
    SolidColor(solid::SolidColorConfig),
}

pub trait Effect{
    type Config: Default;
    fn new(config: Self::Config) -> Self;
    fn apply(&self, colors: &mut Vec<Color>, t:Duration) -> anyhow::Result<()>;
}

pub fn apply_effects(effects: &Vec<impl Effect>, colors: &mut Vec<Color>, t:Duration) -> anyhow::Result<()> {
    for effect in effects {
        effect.apply(colors, t)?;
    }
    Ok(())
}
