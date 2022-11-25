use esp_idf_hal::{gpio::{Pin}, peripheral::{Peripheral, PeripheralRef}, into_ref};

pub mod color;

#[allow(dead_code)]

pub enum LedColorOrder{
    RGB,
    GRB,
}

pub struct StripConfig<'d,T : Pin>{
    pub zero_high_ns: u16,
    pub zero_low_ns: u16,
    pub one_high_ns: u16,
    pub one_low_ns: u16,
    pub reset_ns: u16,
    pub led_count: u16,
    pub led_pin: PeripheralRef<'d, T>,
    pub led_color_order: LedColorOrder,
}

impl<'d, T: Pin> StripConfig<'d, T>{
    pub fn ws2812b(pin:impl Peripheral<P = T> + 'd, pixel_count: u16) -> Self{
        into_ref!(pin);
        StripConfig{
            zero_high_ns: 350,
            zero_low_ns: 800,
            one_high_ns: 800,
            one_low_ns: 350,
            reset_ns: 50000,
            led_count: pixel_count,
            led_pin: pin,
            led_color_order: LedColorOrder::GRB,
        }
    }
    
}

