use std::{thread, sync::{Mutex, Arc}, time::Duration};

use esp_idf_hal::delay::FreeRtos;

use self::{strip::{color::Color, Strip}, effects::EffectConfig};

pub mod strip;
pub mod effects;

// const PIXELCOUNT: u16 = 60;

pub struct NeopixelManager<'a> {
    strip: Arc<strip::Strip<'a>>,
    colors: Arc<Mutex<Vec<Color>>>,
    effects: Vec<EffectConfig>,
}

impl NeopixelManager<'static> {
    pub fn new(strip : Strip<'static>) -> Self {
        let strip = Arc::new(strip);
        let colors = Arc::new(Mutex::new(vec![Color::black(); strip.led_count as usize]));
        Self { strip, colors }
    }

    pub fn run(&self) -> &Self{
        let ccolors = self.colors.clone();
        let sstrip =  self.strip.clone();
        thread::spawn(move|| {
            loop {
                sstrip.send_colors(&ccolors.lock().unwrap()).unwrap();
                FreeRtos::delay_ms(20);
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

