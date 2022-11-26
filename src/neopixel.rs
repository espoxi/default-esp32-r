use std::{thread, time::Duration, sync::{Mutex, Arc}};

use esp_idf_hal::{gpio::{OutputPin, Pin}, rmt::RmtChannel, peripheral::Peripheral};

use self::strip::color::Color;

pub mod strip;

// const PIXELCOUNT: u16 = 60;

pub struct NeopixelManager<'a> {
    pub strip: Arc<strip::Strip<'a>>,
    pub colors: Arc<Mutex<Vec<Color>>>,
}

impl NeopixelManager<'static> {
    pub fn new<T: Pin + OutputPin, C: RmtChannel>(
        pin: impl Peripheral<P = T> +'static,
        rmt_channel: impl Peripheral<P = C> + 'static,
        pixel_count: u16,
    ) -> Self {
        let strip = Arc::new(strip::Strip::ws2812b(pin, rmt_channel, pixel_count));
        let colors = Arc::new(Mutex::new(vec![Color::black(); pixel_count as usize]));
        Self { strip, colors }
    }
    pub fn run_rainbow(& self)-> anyhow::Result<()> {
        let ref mut rainbuf = self.colors.lock().unwrap();
        let size = rainbuf.len();
        for i in 0..size {
            let color = Color::blue().shift_hue((i * 360 / size) as i16);
            rainbuf[i as usize] = color;
        }
        self.strip.send_colors(&rainbuf)?;
        drop(rainbuf);
        let ccolors = self.colors.clone();
        let sstrip =  self.strip.clone();
        thread::spawn(move|| {
            loop {
                let colors = ccolors.lock().unwrap();
                let size = colors.len();
                for i in 0..size {
                    (colors[i as usize]).shift_hue(1);
                }
                sstrip.send_colors(&colors).unwrap();//FIXME
                drop(colors);
                thread::sleep(Duration::from_millis(10));
            }
        });
        Ok(())
    }
    // fn apply_colors(&mut self) -> anyhow::Result<()> {
    //     let colors = &self.colors.lock().unwrap();
    //     self.strip.send_colors(colors)?;
    //     Ok(())
    // }
}
