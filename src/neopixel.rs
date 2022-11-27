use std::{thread, sync::{Mutex, Arc}};

use esp_idf_hal::delay::FreeRtos;

use self::strip::{color::Color, Strip};

pub mod strip;

// const PIXELCOUNT: u16 = 60;

pub struct NeopixelManager<'a> {
    pub strip: Arc<strip::Strip<'a>>,
    pub colors: Arc<Mutex<Vec<Color>>>,
}

impl NeopixelManager<'static> {
    pub fn new(strip : Strip<'static>) -> Self {
        let strip = Arc::new(strip);
        let colors = Arc::new(Mutex::new(vec![Color::black(); strip.led_count as usize]));
        Self { strip, colors }
    }
    pub fn set_rainbow(& self)-> anyhow::Result<()> {
        let ref mut rainbuf = self.colors.lock().unwrap();
        let size = rainbuf.len();
        for i in 0..size {
            let color = *Color::green().shift_hue_deg(i as f32 * 360.0 / size as f32);
            rainbuf[i as usize] = color;
        }
        self.strip.send_colors(&rainbuf)?;
        drop(rainbuf);
        let ccolors = self.colors.clone();
        thread::spawn(move|| {
            loop {
                let mut colors = ccolors.lock().unwrap();
                let size = colors.len();
                for i in 0..size {
                    (colors[i as usize]).shift_hue_deg(5.0);
                }
                drop(colors);
                FreeRtos::delay_ms(20);
            }
        });
        Ok(())
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
