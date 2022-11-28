// #![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]
//#![feature(backtrace)]

mod connection;
mod demos;
mod neopixel;
mod store;

#[allow(unused_imports)]
use std::sync::{Condvar, Mutex};
#[allow(unused_imports)]
use std::{env, sync::Arc, thread, time::*};

#[allow(unused_imports)]
use anyhow::{bail, Result};

use embedded_svc::io::Write;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
#[allow(unused_imports)]
use log::*;

use esp_idf_svc::eventloop::*;
// use esp_idf_svc::sntp;
// use esp_idf_svc::systime::EspSystemTime;
// use esp_idf_svc::timer::*;

use esp_idf_hal::prelude::*;
use neopixel::effects::EffectConfig;

use crate::neopixel::NeopixelManager;
use crate::neopixel::strip::Strip;

// use esp_idf_sys::{self, c_types};

// static store:store::DStore = store::default();

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Get backtraces from anyhow; only works for Xtensa arch currently
    // TODO: No longer working with ESP-IDF 4.3.1+
    //#[cfg(target_arch = "xtensa")]
    //env::set_var("RUST_BACKTRACE", "1");

    #[allow(unused)]
    let peripherals = Peripherals::take().unwrap();
    #[allow(unused)]
    let pins = peripherals.pins;

    let sysloop = EspSystemEventLoop::take()?;
    let store = Arc::new(Mutex::new(store::default()));

    let pixel_count = 30;
    let nm = Arc::new(NeopixelManager::new(Strip::ws2812b(//XXX: 2812(b?)
        pins.gpio14,
        peripherals.rmt.channel1,
        pixel_count,
    )));
    nm.run(20);

    if let Ok(Some(ref mut stored_effects)) = store.lock().unwrap().get::<Vec<EffectConfig>>("effects"){
        nm.effects.lock().unwrap().append(stored_effects);
    }

    let add_route_tx = connection::init(peripherals.modem, sysloop.clone(), store.clone())?;

    let nm2 = nm.clone();
    add_new_route!(add_route_tx; "/effects", Get, move |req|{
        let effects = nm2.effects.lock().unwrap();
        let e = Vec::clone(&effects); //XXX: i wanna be able to just return the lock fast, idk tho if vec clone is faster than parse json, but as u can see here i suppose it is
        drop(effects);
        send_as_json!(req, e)
    });

    let nm3 = nm.clone();
    add_new_route!(add_route_tx; "/effects", Post, move |mut req|{
        let new_effects : Vec<EffectConfig> = parse_req_or_fail_with_message!(req; "couldn't parse effects.. {}");
        store.lock().unwrap().set("effects", &new_effects).unwrap();
        *nm3.effects.lock().unwrap() = new_effects;
        send_as_json!(req, "ok")
    });

    // let _sntp = sntp::EspSntp::new_default()?;
    // info!("SNTP initialized");

    // let (eventloop, _subscription) = test_eventloop()?;

    // let _timer = test_timer(eventloop)?;

    let mut builtin_led = PinDriver::output(pins.gpio2).unwrap();

    loop {
        builtin_led.set_high().unwrap();
        FreeRtos::delay_ms(500);

        builtin_led.set_low().unwrap();
        FreeRtos::delay_ms(500);
    }
}

// fn test_timer(eventloop: EspBackgroundEventLoop) -> Result<EspTimer> {
//     info!("About to schedule a one-shot timer for after 2 seconds");
//     let once_timer = EspTimerService::new()?.timer(|| {
//         info!("One-shot timer triggered");
//     })?;

//     once_timer.after(Duration::from_secs(2))?;

//     thread::sleep(Duration::from_secs(3));

//     info!("About to schedule a periodic timer every five seconds");
//     let periodic_timer = EspTimerService::new()?.timer(move || {
//         info!("Tick from periodic timer");

//         let now = EspSystemTime {}.now();

//         eventloop.post(&EventLoopMessage::new(now), None).unwrap();
//     })?;

//     periodic_timer.every(Duration::from_secs(50))?;

//     Ok(periodic_timer)
// }

// #[derive(Copy, Clone, Debug)]
// struct EventLoopMessage(Duration);

// impl EventLoopMessage {
//     pub fn new(duration: Duration) -> Self {
//         Self(duration)
//     }
// }

// impl EspTypedEventSource for EventLoopMessage {
//     fn source() -> *const c_types::c_char {
//         b"DEMO-SERVICE\0".as_ptr() as *const _
//     }
// }

// impl EspTypedEventSerializer<EventLoopMessage> for EventLoopMessage {
//     fn serialize<R>(
//         event: &EventLoopMessage,
//         f: impl for<'a> FnOnce(&'a EspEventPostData) -> R,
//     ) -> R {
//         f(&unsafe { EspEventPostData::new(Self::source(), Self::event_id(), event) })
//     }
// }

// impl EspTypedEventDeserializer<EventLoopMessage> for EventLoopMessage {
//     fn deserialize<R>(
//         data: &EspEventFetchData,
//         f: &mut impl for<'a> FnMut(&'a EventLoopMessage) -> R,
//     ) -> R {
//         f(unsafe { data.as_payload() })
//     }
// }

// fn test_eventloop() -> Result<(EspBackgroundEventLoop, EspBackgroundSubscription)> {
//     info!("About to start a background event loop");
//     let eventloop = EspBackgroundEventLoop::new(&Default::default())?;

//     info!("About to subscribe to the background event loop");
//     let subscription = eventloop.subscribe(|message: &EventLoopMessage| {
//         info!("Got message from the event loop: {:?}", message.0);
//     })?;

//     Ok((eventloop, subscription))
// }
