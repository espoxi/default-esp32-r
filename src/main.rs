use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_hal::gpio::*;
// use esp_idf_hal::gpio::;
use esp_idf_hal::{peripherals::Peripherals,delay::FreeRtos};

mod connection;
use connection::Connection;

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();


    let _conn : Option<Connection<'static>> = match Connection::new(peripherals.modem) {
        Ok(mut c) => {match c.start_service(){
            Ok(_) => Some(c),
            Err(e) => {
                println!("Error starting service: {}", e);
                None
            }
        }},
        Err(e) => {println!("Failed to start server: {:?}", e); None}
    };

    let mut internal_led = PinDriver::output(peripherals.pins.gpio2).unwrap();

    loop {
        internal_led.set_high().unwrap();
        // we are sleeping here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(500);

        internal_led.set_low().unwrap();
        FreeRtos::delay_ms(500);
    }
}
