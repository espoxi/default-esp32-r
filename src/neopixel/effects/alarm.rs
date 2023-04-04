use std::{ops::Range, time::Duration};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};

use crate::neopixel::strip::color::FColor;

use super::{hue::HueShiftEffect, solid::SolidColorEffect, strobo::StroboEffect, Effect};

pub struct AlarmEffect;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmConfig {
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub at_ms_since_1970: Duration,
    pub alarm_type: AlarmType,
    pub range: Range<u16>,
}

impl Default for AlarmConfig {
    fn default() -> Self {
        Self {
            at_ms_since_1970: Duration::from_secs(2680471881),
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
        colors: &mut Vec<FColor>,
        dt: Duration,
        rt: Option<Duration>,
    ) -> anyhow::Result<()> {
        if let Some(rt) = rt {
            if rt > config.at_ms_since_1970 {
                //TODO: play actual alarm

                let seconds_to_alarm = config.at_ms_since_1970.as_secs_f32() - rt.as_secs_f32();
                match config.alarm_type {
                    AlarmType::Sunrise => {
                        //fade in red color from 30s to 10s before alarm
                        let red = if seconds_to_alarm > 30.0 {
                            0.0
                        } else if seconds_to_alarm > 10.0 {
                            1.0 - (seconds_to_alarm - 10.0) / 20.0
                        } else {
                            1.0
                        };
                        //fade in green color from 20s to 5s before alarm
                        let green = if seconds_to_alarm > 20.0 {
                            0.0
                        } else if seconds_to_alarm > 5.0 {
                            1.0 - (seconds_to_alarm - 5.0) / 15.0
                        } else {
                            1.0
                        };
                        //fade in blue color from 10s to 0s before alarm
                        let blue = if seconds_to_alarm > 10.0 {
                            0.0
                        } else {
                            1.0 - seconds_to_alarm / 10.0
                        };
                        SolidColorEffect::apply(
                            &super::solid::SolidColorConfig {
                                color: FColor::new(red, green, blue),
                                range: config.range.clone(),
                            },
                            colors,
                            dt,
                            Some(rt),
                        )?;
                    }
                    AlarmType::Silvester => {
                        //countdown to 0 ; blink white every second (black in between blinks)
                        if seconds_to_alarm < 0.0 {
                            //its time ; friggn party
                            //solid base color for hue-shift to make a rainbow
                            SolidColorEffect::apply(
                                &super::solid::SolidColorConfig {
                                    color: FColor::red(),
                                    range: config.range.clone(),
                                },
                                colors,
                                dt,
                                Some(rt),
                            )?;
                            HueShiftEffect::apply(
                                &super::hue::HueShiftConfig {
                                    range: config.range.clone(),
                                    degrees_per_led: 12.0, //full rotataion in 30LEDs (1m)
                                    degrees_per_second: 60.0, //full rotation in 6s (rather fast)
                                },
                                colors,
                                dt,
                                Some(rt),
                            )?;
                        } else {
                            if seconds_to_alarm < 3.0 && seconds_to_alarm % 1.0 < 0.4 {
                                //red afterglow in last 3s
                                SolidColorEffect::apply(
                                    &super::solid::SolidColorConfig {
                                        color: FColor::red(),
                                        range: config.range.clone(),
                                    },
                                    colors,
                                    dt,
                                    Some(rt),
                                )?;
                            }
                            if seconds_to_alarm % 1.0 < 0.2 {
                                //blink white
                                SolidColorEffect::apply(
                                    &super::solid::SolidColorConfig {
                                        color: FColor::white(),
                                        range: config.range.clone(),
                                    },
                                    colors,
                                    dt,
                                    Some(rt),
                                )?;
                            } else {
                                //black
                                SolidColorEffect::apply(
                                    &super::solid::SolidColorConfig {
                                        color: FColor::black(),
                                        range: config.range.clone(),
                                    },
                                    colors,
                                    dt,
                                    Some(rt),
                                )?;
                            }
                        }
                    }
                    AlarmType::Strobo => {
                        if rt < config.at_ms_since_1970 + Duration::from_secs(30) {
                            //play strobo for 30s
                            StroboEffect::apply(
                                &super::strobo::StroboConfig {
                                    frequency_hz: 2.0,
                                    range: config.range.clone(),
                                },
                                colors,
                                dt,
                                Some(rt),
                            )?;
                        }
                    }
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
