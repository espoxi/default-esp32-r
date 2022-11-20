use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_hal::gpio::*;
// use esp_idf_hal::gpio::;
use esp_idf_hal::{peripherals::Peripherals,delay::FreeRtos};

mod connection;
use connection::Connection;

mod eventhandler;

use common::store;

fn main() {    
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();
    inner_main()
}

fn inner_main<'a>(){
    let peripherals = Peripherals::take().unwrap();
    let mut store = store::default();//TODO: static? es muss mindestens genauso lange leben wie die conn

    let _conn : Option<Connection<'a>> = match Connection::new(peripherals.modem, &store) {
        Ok(mut c) => {match c.start_service(&mut store){
            Ok(_) => Some(c),
            Err(e) => {
                println!("Error starting service: {}", e);
                None
            }
        }},
        Err(e) => {println!("Failed to start server: {:?}", e); None},
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