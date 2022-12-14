use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use esp_idf_hal::delay::FreeRtos;

use self::{
    effects::EffectConfig,
    strip::{color::Color, Strip},
};

pub mod effects;
pub mod strip;

// const PIXELCOUNT: u16 = 60;

pub struct NeopixelManager<'a> {
    strip: Arc<strip::Strip<'a>>,
    colors: Arc<Mutex<Vec<Color>>>,
    pub effects: Arc<Mutex<Vec<EffectConfig>>>,
}

impl NeopixelManager<'static> {
    pub fn new(strip: Strip<'static>) -> Self {
        let strip = Arc::new(strip);
        let colors = Arc::new(Mutex::new(vec![Color::black(); strip.led_count as usize]));
        let effects = Arc::new(Mutex::new(Vec::new()));
        Self {
            strip,
            colors,
            effects,
        }
    }

    ///mspf = milliseconds per frame = 1000 / fps
    pub fn run(&self, mspf: u32) -> &Self {
        let ccolors = self.colors.clone();
        let sstrip = self.strip.clone();
        let eeffects = self.effects.clone();
        thread::spawn(move || {
            let s = Instant::now();
            loop {
                let effects = eeffects.lock().unwrap();
                let mut colors = ccolors.lock().unwrap();
                effects::apply_effects(&effects, &mut colors, Instant::now() - s).unwrap();
                // println!("applied effects effects: {:?}", effects);
                drop(effects);
                sstrip.send_colors(&colors).unwrap();
                drop(colors);
                FreeRtos::delay_ms(mspf);
            }
        });
        self
    }
    // fn apply_colors(&mut self) -> anyhow::Result<()> {
    //     let colors = &self.colors.lock().unwrap();
    //     self.strip.send_colors(colors)?;
    //     Ok(())
    // }
}
