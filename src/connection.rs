use std::{
    sync::{Arc, Mutex},
    thread,
};

use anyhow::{bail, Result};
use embedded_svc::ipv4;
use esp_idf_hal::{delay::FreeRtos, peripheral};
use esp_idf_svc::{eventloop::EspSystemEventLoop, ping};
use log::{info, warn};

pub mod client;
pub mod server;
pub mod wifi;

use wifi::{Creds, Wlan};

use crate::{
    connection::server as s,
    store::{DStore, SelfStorable},
};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

pub fn init(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static + std::marker::Send,
    sysloop: EspSystemEventLoop,
    sstore: Arc<Mutex<DStore>>,
) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    thread::Builder::new().stack_size(6 * 1024).spawn(move || {
        info!("Initializing wifi...");
        let mut wifi = match Wlan::start(modem, sysloop) {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to start wifi: {}", e);
                let _ = tx.send(Err(e));
                return;
            }
        };

        let ssstore = sstore.lock().unwrap();

        info!("Connecting to stored wifi...");
        if let Ok(Some(creds)) = ssstore.get("client_creds") {
            match wifi.connect_to(creds) {
                Err(e) => warn!("Failed to connect to stored wifi: {}", e),
                Ok(_) => info!("Connected to stored wifi"),
            };
        } else {
            info!("No stored wifi credentials, we will start our own access point");
            match wifi.host_as(match ssstore.get("ap_creds") {
                Ok(Some(creds)) => creds,
                _ => Creds {
                    ssid: CONFIG.wifi_ssid.into(),
                    psk: CONFIG.wifi_psk.into(),
                },
            }) {
                Ok(_) => info!("Wifi started as host"),
                Err(e) => warn!("Wifi hosting failed: {}", e),
            };
        }
        drop(ssstore);

        info!("Initializing http server...");
        let mut server = match server::init_server() {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to start http server: {}", e);
                let _ = tx.send(Err(e));
                return;
            }
        };

        let (inner_tx, inner_rx) = std::sync::mpsc::channel::<ConnectionEvent>();

        s::add_connect_route(&mut server, inner_tx.clone()).unwrap();
        s::add_rename_route(&mut server, inner_tx.clone()).unwrap();
        let _ = tx.send(Ok(()));
        loop {
            FreeRtos::delay_ms(100);
            match inner_rx.recv() {
                Ok(ConnectionEvent::ConnectToWifi(creds)) => {
                    info!("Connecting to wifi...");
                    let mut ssstore = sstore.lock().unwrap();
                    ssstore.set("client_creds", &creds).unwrap();
                    match wifi.connect_to(creds) {
                        Ok(_) => info!("Connected to wifi"),
                        Err(e) => warn!("Failed to connect to wifi: {}", e),
                    };
                }
                Ok(ConnectionEvent::HostAs(creds)) => {
                    info!("Starting wifi as host...");
                    let mut ssstore = sstore.lock().unwrap();
                    ssstore.set("ap_creds", &creds).unwrap();
                    match wifi.host_as(creds) {
                        Ok(_) => info!("Wifi started as host"),
                        Err(e) => warn!("Wifi hosting failed: {}", e),
                    };
                }
                Err(_) => warn!("Connection event channel closed"),
            }
        }
    })?;
    rx.recv().unwrap()
}

pub enum ConnectionEvent {
    ConnectToWifi(Creds),
    HostAs(Creds),
}

pub fn ping(ip: ipv4::Ipv4Addr) -> Result<()> {
    info!("About to do some pings for {:?}", ip);

    let ping_summary = ping::EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!("Pinging IP {} resulted in timeouts", ip);
    }

    info!("Pinging done");

    Ok(())
}
