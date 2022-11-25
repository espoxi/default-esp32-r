use std::{thread, sync::Arc};

use anyhow::{bail, Result};
use embedded_svc::ipv4;
use esp_idf_hal::peripheral;
use esp_idf_svc::{eventloop::EspSystemEventLoop, ping};
use log::{info, warn};

pub mod client;
pub mod server;
pub mod wifi;

use wifi::{Creds, Wlan};

use crate::{
    connection::server::add_connect_route,
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
    sstore: Arc<DStore>,
) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move|| {
        info!("Initializing wifi...");
        let mut wifi = match Wlan::start(modem, sysloop) {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to start wifi: {}", e);
                tx.send(Err(e));
                return;
            }
        };

        info!("Connecting to stored wifi...");
        if let Ok(creds) = Creds::from_store(&sstore.clone()) {
            match wifi.connect_to(creds) {
                Err(e) => warn!("Failed to connect to stored wifi: {}", e),
                Ok(_) => info!("Connected to stored wifi"),
            };
        } else {
            info!("No stored wifi credentials, we will start our own access point");
            match wifi.host_as(Creds {
                ssid: CONFIG.wifi_ssid.into(),
                psk: CONFIG.wifi_psk.into(),
            }) {
                Ok(_) => info!("Wifi started as host"),
                Err(e) => warn!("Wifi hosting failed: {}", e),
            };
        }

        info!("Initializing http server...");
        let mut server = match server::init_server() {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to start http server: {}", e);
                tx.send(Err(e));
                return;
            }
        };

        add_connect_route(&mut server).unwrap();
        tx.send(Ok(()));
        loop {}
    });

    Ok(())
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
