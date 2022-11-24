// #![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]
//#![feature(backtrace)]

mod connection;
mod demos;
mod store;

use std::sync::{Condvar, Mutex};
use std::{env, sync::Arc, thread, time::*};

use anyhow::{bail, Result};

use log::*;

use esp_idf_svc::eventloop::*;
use esp_idf_svc::sntp;
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::timer::*;

use esp_idf_hal::adc;
use esp_idf_hal::prelude::*;

use esp_idf_sys::{self, c_types};

use crate::connection::server::init_httpd;
use crate::connection::wifi::Creds;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

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

    #[allow(unused)]
    let sysloop = EspSystemEventLoop::take()?;

    #[allow(clippy::redundant_clone)]
    #[cfg(not(feature = "qemu"))]
    let mut wifi = connection::wifi::Wlan::start(peripherals.modem, sysloop.clone())?;
    match wifi.host_as(Creds {
        ssid: CONFIG.wifi_ssid.into(),
        psk: CONFIG.wifi_psk.into(),
    }) {
        Ok(_) => info!("Wifi started as host"),
        Err(e) => warn!("Wifi hosting failed: {}", e),
    };
    wifi.connect_to(Creds {
        ssid: "----".into(),
        psk: "----".into(),
    })?;

    demos::demo_all()?;

    let _sntp = sntp::EspSntp::new_default()?;
    info!("SNTP initialized");

    let (eventloop, _subscription) = test_eventloop()?;

    let _timer = test_timer(eventloop)?;

    #[cfg(feature = "experimental")]
    let mutex = Arc::new((Mutex::new(None), Condvar::new()));

    let httpd = init_httpd(mutex.clone())?;

    // let mut wait = mutex.0.lock().unwrap();

    // #[cfg(esp32)]
    // let mut hall_sensor = peripherals.hall_sensor;

    // #[cfg(esp32)]
    // let adc_pin = pins.gpio34;
    // #[cfg(any(esp32s2, esp32s3, esp32c3))]
    // let adc_pin = pins.gpio2;

    // let mut a2 = adc::AdcChannelDriver::<_, adc::Atten11dB<adc::ADC1>>::new(adc_pin)?;

    // let mut powered_adc1 = adc::AdcDriver::new(
    //     peripherals.adc1,
    //     &adc::config::Config::new().calibration(true),
    // )?;

    // #[allow(unused)]
    // let cycles = loop {
    //     if let Some(cycles) = *wait {
    //         break cycles;
    //     } else {
    //         wait = mutex
    //             .1
    //             .wait_timeout(wait, Duration::from_secs(1))
    //             .unwrap()
    //             .0;

    //         // #[cfg(esp32)]
    //         // log::info!(
    //         //     "Hall sensor reading: {}mV",
    //         //     powered_adc1.read_hall(&mut hall_sensor).unwrap()
    //         // );
    //         // log::info!(
    //         //     "A2 sensor reading: {}mV",
    //         //     powered_adc1.read(&mut a2).unwrap()
    //         // );
    //     }
    // };

    for s in 0..3 {
        info!("Shutting down in {} secs", 3 - s);
        thread::sleep(Duration::from_secs(1));
    }

    drop(httpd);
    info!("Httpd stopped");

    #[cfg(not(feature = "qemu"))]
    {
        drop(wifi);
        info!("Wifi stopped");
    }

    Ok(())
}

fn test_timer(eventloop: EspBackgroundEventLoop) -> Result<EspTimer> {
    info!("About to schedule a one-shot timer for after 2 seconds");
    let once_timer = EspTimerService::new()?.timer(|| {
        info!("One-shot timer triggered");
    })?;

    once_timer.after(Duration::from_secs(2))?;

    thread::sleep(Duration::from_secs(3));

    info!("About to schedule a periodic timer every five seconds");
    let periodic_timer = EspTimerService::new()?.timer(move || {
        info!("Tick from periodic timer");

        let now = EspSystemTime {}.now();

        eventloop.post(&EventLoopMessage::new(now), None).unwrap();
    })?;

    periodic_timer.every(Duration::from_secs(5))?;

    Ok(periodic_timer)
}

#[derive(Copy, Clone, Debug)]
struct EventLoopMessage(Duration);

impl EventLoopMessage {
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }
}

impl EspTypedEventSource for EventLoopMessage {
    fn source() -> *const c_types::c_char {
        b"DEMO-SERVICE\0".as_ptr() as *const _
    }
}

impl EspTypedEventSerializer<EventLoopMessage> for EventLoopMessage {
    fn serialize<R>(
        event: &EventLoopMessage,
        f: impl for<'a> FnOnce(&'a EspEventPostData) -> R,
    ) -> R {
        f(&unsafe { EspEventPostData::new(Self::source(), Self::event_id(), event) })
    }
}

impl EspTypedEventDeserializer<EventLoopMessage> for EventLoopMessage {
    fn deserialize<R>(
        data: &EspEventFetchData,
        f: &mut impl for<'a> FnMut(&'a EventLoopMessage) -> R,
    ) -> R {
        f(unsafe { data.as_payload() })
    }
}

fn test_eventloop() -> Result<(EspBackgroundEventLoop, EspBackgroundSubscription)> {
    info!("About to start a background event loop");
    let eventloop = EspBackgroundEventLoop::new(&Default::default())?;

    info!("About to subscribe to the background event loop");
    let subscription = eventloop.subscribe(|message: &EventLoopMessage| {
        info!("Got message from the event loop: {:?}", message.0);
    })?;

    Ok((eventloop, subscription))
}
