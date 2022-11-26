use std::{thread, time::Duration, sync::{Mutex, Arc}};

use esp_idf_hal::{gpio::{OutputPin, Pin}, rmt::RmtChannel, peripheral::Peripheral};

use self::strip::color::Color;

pub mod strip;

const PIXELCOUNT: u16 = 60;

pub struct NeopixelManager<'a> {
    pub strip: strip::Strip<'a>,
    pub colors: Arc<Mutex<[Color; PIXELCOUNT as usize]>>,
}

impl<'a> NeopixelManager<'a> {
    pub fn new<T: Pin + OutputPin, C: RmtChannel>(
        pin: impl Peripheral<P = T> + 'a,
        rmt_channel: impl Peripheral<P = C> + 'a,
    ) -> Self {
        let strip = strip::Strip::ws2812b(pin, rmt_channel, PIXELCOUNT);
        let colors = Arc::new(Mutex::new([Color::black(); PIXELCOUNT as usize]));
        Self { strip, colors }
    }
    pub fn run_rainbow(&mut self)-> anyhow::Result<()> {
        let rainbuf = [Color::green(); PIXELCOUNT as usize];
        for i in 0..PIXELCOUNT {
            (rainbuf[i as usize]).shift_hue((i * 360 / PIXELCOUNT) as i16);
        }
        *self.colors.lock().unwrap() = rainbuf;
        thread::spawn(|| {
            loop {
                let colors = self.colors.lock().unwrap();
                for i in 0..PIXELCOUNT {
                    (colors[i as usize]).shift_hue(1);
                }
                self.strip.send_colors(&colors).unwrap();//FIXME
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
