

use std::time::Duration;

use esp_idf_hal::{
    delay::Ets,
    gpio::{OutputPin, Pin},
    peripheral::{Peripheral},
    rmt::{config::TransmitConfig, RmtChannel, TxRmtDriver, Pulse, PinState, VariableLengthSignal},
};
use anyhow::{Result};

pub mod color;

#[allow(dead_code)]

pub enum LedColorOrder {
    RGB,
    GRB,
}

pub struct Strip<'d> {
    pub zero_high_ns: u16,
    pub zero_low_ns: u16,
    pub one_high_ns: u16,
    pub one_low_ns: u16,
    pub reset_ns: u16,
    pub led_count: u16,
    rmt: TxRmtDriver<'d>,
    pub led_color_order: LedColorOrder,
}

impl<'d> Strip<'d> {
    pub fn ws2812b<T: Pin + OutputPin, C: RmtChannel>(
        pin: impl Peripheral<P = T> + 'd,
        rmt_channel: impl Peripheral<P = C> + 'd,
        pixel_count: u16,
    ) -> Self {
        let config = TransmitConfig::new().clock_divider(1);
        let mut tx = TxRmtDriver::new(rmt_channel, pin, &config).unwrap();
        Self {
            zero_high_ns: 400,
            zero_low_ns: 850,
            one_high_ns: 800,
            one_low_ns: 450,
            reset_ns: 55000,
            led_count: pixel_count,
            rmt: tx,
            led_color_order: LedColorOrder::GRB,
        }
    }
    pub fn send_colors(&mut self, colors: &[color::Color])-> Result<()> {
        let ticks_hz = self.rmt.counter_clock()?;
        let t0h = Pulse::new_with_duration(ticks_hz, PinState::High, &ns(self.zero_high_ns))?;
        let t0l = Pulse::new_with_duration(ticks_hz, PinState::Low, &ns(self.zero_low_ns))?;
        let t1h = Pulse::new_with_duration(ticks_hz, PinState::High, &ns(self.one_high_ns))?;
        let t1l = Pulse::new_with_duration(ticks_hz, PinState::Low, &ns(self.one_low_ns))?;

        let mut signal = VariableLengthSignal::with_capacity(24*colors.len());
        for color in colors {
            // for bit in color.to_bit_iter(self.led_color_order) {
            //     let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
            //     signal.push(&(high_pulse, low_pulse))?;
            // }
            signal.push(color.to_bit_iter(&self.led_color_order).map(|bit|{
                if bit {&[t1h, t1l]}  else {&[t0h, t0l]}//FIXME
            }).flatten());
        }
        self.rmt.start(signal)?;
        Ets::delay_us((self.reset_ns / 1000) as u32);
        Ok(())
    }
}

fn ns(nanos: u16) -> Duration {
    Duration::from_nanos(nanos as u64)
}