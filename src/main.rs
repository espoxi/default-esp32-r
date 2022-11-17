use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_hal::gpio::*;
// use esp_idf_hal::gpio::;
use esp_idf_hal::{peripherals::Peripherals,delay::FreeRtos};

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio4)?;

    loop {
        // led.set_high()?;
        // // we are sleeping here to make sure the watchdog isn't triggered
        // FreeRtos::delay_ms(1000);

        // led.set_low()?;
        // FreeRtos::delay_ms(1000);
    }
}
