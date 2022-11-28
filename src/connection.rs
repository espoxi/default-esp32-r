use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};

use anyhow::{bail, Result};
use embedded_svc::ipv4;
use esp_idf_hal::peripheral;
use esp_idf_svc::{eventloop::EspSystemEventLoop, ping};
use log::{info, warn};

pub mod client;
pub mod server;
pub mod wifi;

use wifi::{Creds, Wlan};

use crate::{connection::server as s, store::DStore};

use server::RouteData;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

#[macro_export]
macro_rules! add_new_route {
    ($sender:expr; $uri:expr, $method:ident, $handler:expr) => {
        $sender
            .send(crate::connection::ConnectionRelevantEvent::Route(
                crate::connection::server::RouteData::new(
                    $uri,
                    embedded_svc::http::server::Method::$method,
                    $handler,
                ),
            ))
            .unwrap();
    };
}

pub enum ConnectionRelevantEvent {
    Wifi(ConnectionEvent),
    Route(RouteData),
}

pub fn init(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static + std::marker::Send,
    sysloop: EspSystemEventLoop,
    sstore: Arc<Mutex<DStore>>,
) -> Result<Sender<ConnectionRelevantEvent>> {
    let (succesful_wifi_connection_tx, succesful_wifi_connection_rx) = std::sync::mpsc::channel();
    let (ttx, rx) = std::sync::mpsc::channel::<ConnectionRelevantEvent>();
    let tx = ttx.clone();
    thread::Builder::new().stack_size(6 * 1024).spawn(move || {
        info!("Initializing wifi...");
        let mut wifi = match Wlan::start(modem, sysloop) {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to start wifi: {}", e);
                let _ = succesful_wifi_connection_tx.send(Err(e));
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
                let _ = succesful_wifi_connection_tx.send(Err(e));
                return;
            }
        };

        let (connection_tx, connection_rx) = std::sync::mpsc::channel();
        s::add_connect_route(&mut server, tx.clone(), connection_rx).unwrap();
        s::add_rename_route(&mut server, tx.clone()).unwrap();
        let _ = succesful_wifi_connection_tx.send(Ok(()));
        loop {
            // FreeRtos::delay_ms(100); //not needed since we are blocking on the rx (no busy waiting, and therefore no polling delay needed)
            match rx.recv() {
                Ok(ConnectionRelevantEvent::Wifi(event)) => match event {
                    ConnectionEvent::ConnectToWifi(creds) => {
                        info!("Connecting to wifi...");
                        let mut ssstore = sstore.lock().unwrap();
                        ssstore.set("client_creds", &creds).unwrap();
                        match wifi.connect_to(creds) {
                            Ok(address) => {
                                info!("Connected to wifi as {}", address);
                                let _ = connection_tx.send(Ok(address));
                            }
                            Err(e) => {
                                warn!("Failed to connect to wifi: {}", e);
                                let _ = connection_tx.send(Err(e));
                            }
                        };
                    }
                    ConnectionEvent::HostAs(creds) => {
                        info!("Starting wifi as host...");
                        let mut ssstore = sstore.lock().unwrap();
                        ssstore.set("ap_creds", &creds).unwrap();
                        match wifi.host_as(creds) {
                            Ok(_) => info!("Wifi started as host"),
                            Err(e) => warn!("Wifi hosting failed: {}", e),
                        };
                    }
                },
                Ok(ConnectionRelevantEvent::Route(route_data)) => {
                    match s::add_new_route(&mut server, route_data) {
                        Ok(_) => info!("Added new route"),
                        Err(e) => warn!("Failed to add new route: {}", e),
                    };
                }
                Err(_) => {
                    warn!("Route data channel closed");
                    break;
                }
            }
        }
    })?;
    succesful_wifi_connection_rx.recv()??;
    Ok(ttx)
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
