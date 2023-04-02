use std::{ops::Range, time::Duration};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

use crate::neopixel::strip::color::Color;

use super::Effect;

pub struct AlarmEffect;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmConfig {
    #[serde_as(as = "DurationSeconds<f64>")]
    pub at_s_since_1970: Duration,
    pub alarm_type: AlarmType,
    pub range: Range<u16>,
}

impl Default for AlarmConfig {
    fn default() -> Self {
        Self {
            at_s_since_1970: Duration::from_secs(2680471881),
            alarm_type: AlarmType::Sunrise,
            range: 0..30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlarmType {
    Sunrise,
    Silvester,
    Strobo,
}

impl Effect for AlarmEffect {
    type Config = AlarmConfig;
    fn apply(
        config: &Self::Config,
        colors: &mut Vec<Color>,
        dt: Duration,
        rt: Option<Duration>,
    ) -> anyhow::Result<()> {
        if let Some(rt) = rt {
            if rt > config.at_s_since_1970 {
                //TODO: play actual alarm
                match config.alarm_type {
                    AlarmType::Sunrise => {}
                    AlarmType::Silvester => {}
                    AlarmType::Strobo => {}
                }
            }
        }
        // for i in config.range.clone() {
        //     // (colors[i as usize]).shift_hue_deg(
        //     //     config.degrees_per_second * t.as_secs_f32() + config.degrees_per_led * i as f32,
        //     // );
        // }
        Ok(())
    }
}
