// #![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]
//#![feature(backtrace)]

mod connection;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

#[cfg(all(feature = "qemu", not(esp32)))]
compile_error!("The `qemu` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "ip101", not(esp32)))]
compile_error!("The `ip101` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "kaluga", not(esp32s2)))]
compile_error!("The `kaluga` feature can only be built for the `xtensa-esp32s2-espidf` target.");

#[cfg(all(feature = "ttgo", not(esp32)))]
compile_error!("The `ttgo` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "heltec", not(esp32)))]
compile_error!("The `heltec` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "esp32s3_usb_otg", not(esp32s3)))]
compile_error!(
    "The `esp32s3_usb_otg` feature can only be built for the `xtensa-esp32s3-espidf` target."
);

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Condvar, Mutex};
use std::{ env,  sync::Arc, thread, time::*};

use anyhow::{bail, Result};

use log::*;

use embedded_svc::ipv4;

use esp_idf_svc::eventloop::*;
use esp_idf_svc::ping;
use esp_idf_svc::sntp;
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::timer::*;

use esp_idf_hal::adc;
use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::prelude::*;

use esp_idf_sys::{self, c_types};


#[cfg(esp32s2)]
include!(env!("EMBUILD_GENERATED_SYMBOLS_FILE"));

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
    #[allow(unused_mut)]
    let mut wifi = connection::wifi::test_wifi(peripherals.modem, sysloop.clone())?;

    test_tcp()?;

    test_tcp_bind()?;

    let _sntp = sntp::EspSntp::new_default()?;
    info!("SNTP initialized");

    let (eventloop, _subscription) = test_eventloop()?;

    let _timer = test_timer(eventloop)?;

    #[cfg(feature = "experimental")]
    experimental::test()?;

    let mutex = Arc::new((Mutex::new(None), Condvar::new()));

    let httpd = httpd(mutex.clone())?;

    let mut wait = mutex.0.lock().unwrap();

    #[cfg(esp32)]
    let mut hall_sensor = peripherals.hall_sensor;

    #[cfg(esp32)]
    let adc_pin = pins.gpio34;
    #[cfg(any(esp32s2, esp32s3, esp32c3))]
    let adc_pin = pins.gpio2;

    let mut a2 = adc::AdcChannelDriver::<_, adc::Atten11dB<adc::ADC1>>::new(adc_pin)?;

    let mut powered_adc1 = adc::AdcDriver::new(
        peripherals.adc1,
        &adc::config::Config::new().calibration(true),
    )?;

    #[allow(unused)]
    let cycles = loop {
        if let Some(cycles) = *wait {
            break cycles;
        } else {
            wait = mutex
                .1
                .wait_timeout(wait, Duration::from_secs(1))
                .unwrap()
                .0;

            #[cfg(esp32)]
            log::info!(
                "Hall sensor reading: {}mV",
                powered_adc1.read_hall(&mut hall_sensor).unwrap()
            );
            log::info!(
                "A2 sensor reading: {}mV",
                powered_adc1.read(&mut a2).unwrap()
            );
        }
    };

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

    #[cfg(any(feature = "qemu", feature = "w5500", feature = "ip101"))]
    {
        drop(eth);
        info!("Eth stopped");
    }

    Ok(())
}



fn test_tcp() -> Result<()> {
    info!("About to open a TCP connection to 1.1.1.1 port 80");

    let mut stream = TcpStream::connect("one.one.one.one:80")?;

    let err = stream.try_clone();
    if let Err(err) = err {
        info!(
            "Duplication of file descriptors does not work (yet) on the ESP-IDF, as expected: {}",
            err
        );
    }

    stream.write_all("GET / HTTP/1.0\n\n".as_bytes())?;

    let mut result = Vec::new();

    stream.read_to_end(&mut result)?;

    info!(
        "1.1.1.1 returned:\n=================\n{}\n=================\nSince it returned something, all is OK",
        std::str::from_utf8(&result)?);

    Ok(())
}

fn test_tcp_bind() -> Result<()> {
    fn test_tcp_bind_accept() -> Result<()> {
        info!("About to bind a simple echo service to port 8080");

        let listener = TcpListener::bind("0.0.0.0:8080")?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!("Accepted client");

                    thread::spawn(move || {
                        test_tcp_bind_handle_client(stream);
                    });
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }

        unreachable!()
    }

    fn test_tcp_bind_handle_client(mut stream: TcpStream) {
        // read 20 bytes at a time from stream echoing back to stream
        loop {
            let mut read = [0; 128];

            match stream.read(&mut read) {
                Ok(n) => {
                    if n == 0 {
                        // connection was closed
                        break;
                    }
                    stream.write_all(&read[0..n]).unwrap();
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }

    thread::spawn(|| test_tcp_bind_accept().unwrap());

    Ok(())
}

fn test_timer(
    eventloop: EspBackgroundEventLoop,
) -> Result<EspTimer> {

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

#[cfg(feature = "experimental")]
mod experimental {
    use super::{thread, TcpListener, TcpStream};
    use log::info;


    pub fn test() -> anyhow::Result<()> {
        #[cfg(not(esp_idf_version = "4.3"))]
        test_tcp_bind_async()?;

        test_https_client()?;

        Ok(())
    }

    #[cfg(not(esp_idf_version = "4.3"))]
    fn test_tcp_bind_async() -> anyhow::Result<()> {
        async fn test_tcp_bind() -> smol::io::Result<()> {
            /// Echoes messages from the client back to it.
            async fn echo(stream: smol::Async<TcpStream>) -> smol::io::Result<()> {
                smol::io::copy(&stream, &mut &stream).await?;
                Ok(())
            }

            // Create a listener.
            let listener = smol::Async::<TcpListener>::bind(([0, 0, 0, 0], 8081))?;

            // Accept clients in a loop.
            loop {
                let (stream, peer_addr) = listener.accept().await?;
                info!("Accepted client: {}", peer_addr);

                // Spawn a task that echoes messages from the client back to it.
                smol::spawn(echo(stream)).detach();
            }
        }

        info!("About to bind a simple echo service to port 8081 using async (smol-rs)!");

        #[allow(clippy::needless_update)]
        {
            esp_idf_sys::esp!(unsafe {
                esp_idf_sys::esp_vfs_eventfd_register(&esp_idf_sys::esp_vfs_eventfd_config_t {
                    max_fds: 5,
                    ..Default::default()
                })
            })?;
        }

        thread::Builder::new().stack_size(4096).spawn(move || {
            smol::block_on(test_tcp_bind()).unwrap();
        })?;

        Ok(())
    }

    fn test_https_client() -> anyhow::Result<()> {
        use embedded_svc::http::{client::*, };
        use embedded_svc::io::Read;
        use embedded_svc::utils::io;
        use esp_idf_svc::http::client::*;

        let url = String::from("https://google.com");

        info!("About to fetch content from {}", url);

        let mut client = Client::wrap(EspHttpConnection::new(&Configuration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

            ..Default::default()
        })?);

        let mut response = client.get(&url)?.submit()?;

        let mut body = [0_u8; 3048];

        let read = io::try_read_full(&mut response, &mut body).map_err(|err| err.0)?;

        info!(
            "Body (truncated to 3K):\n{:?}",
            String::from_utf8_lossy(&body[..read]).into_owned()
        );

        // Complete the response
        while response.read(&mut body)? > 0 {}

        Ok(())
    }
}



#[allow(unused_variables)]
#[cfg(feature = "experimental")]
fn httpd(
    mutex: Arc<(Mutex<Option<u32>>, Condvar)>,
) -> Result<esp_idf_svc::http::server::EspHttpServer> {
    use embedded_svc::http::server::{Method};
    use embedded_svc::io::Write;

    let mut server = esp_idf_svc::http::server::EspHttpServer::new(&Default::default())?;

    server
        .fn_handler("/", Method::Get, |req| {
            req.into_ok_response()?
                .write_all("Hello from Rust!".as_bytes())?;

            Ok(())
        })?
        .fn_handler("/foo", Method::Get, |_req| {
            Result::Err("Boo, something happened!".into())
        })?
        .fn_handler("/bar", Method::Get, |req| {
            req.into_response(403, Some("No permissions"), &[])?
                .write_all("You have no permissions to access this page".as_bytes())?;

            Ok(())
        })?
        .fn_handler("/panic", Method::Get, |_req| {
            panic!("User requested a panic!")
        })?;

    Ok(server)
}


fn ping(ip: ipv4::Ipv4Addr) -> Result<()> {
    info!("About to do some pings for {:?}", ip);

    let ping_summary = ping::EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!("Pinging IP {} resulted in timeouts", ip);
    }

    info!("Pinging done");

    Ok(())
}
