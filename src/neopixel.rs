use std::{thread, time::Duration, sync::{Mutex, Arc}};

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
            let color = *Color::green().shift_hue((i * 360 / size) as i16);
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
                    (colors[i as usize]).shift_hue(10);
                }
                drop(colors);
                thread::sleep(Duration::from_millis(100));
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
                thread::sleep(Duration::from_millis(100));
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
