use common::events::ApiEvent;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use anyhow::bail;
use common::store::DStore;

use esp_idf_hal::modem::Modem;
use esp_idf_svc::http::server::{
    Configuration, EspHttpServer,
};
use std::str;
use std::sync::mpsc::channel;

mod wifi;
use wifi::Wifi;

// use crate::connection::api::ApiEvent;

mod api;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

pub struct Connection<'a> {
    wifi: Option<wifi::Wifi<'a>>,
    server: EspHttpServer,
}


impl<'a> Connection<'a> {
    pub(crate) fn start_service(&mut self, store: &mut DStore) -> anyhow::Result<()> {
        let server = &mut self.server;

        let (tx, rx) = channel::<ApiEvent>();

        api::init(server, tx);

        let (tx2, rx2) = channel();

        println!("waiting for wifi credentials");
        let (ssid, psk) = rx.recv().unwrap();
        match self.wifi {
            Some(ref mut w) => {
                let creds = wifi::Creds::new(ssid, psk);
                match creds.store_in(store) {
                    Ok(_) => println!("stored wifi credentials"),
                    Err(e) => println!("failed to store wifi credentials: {}", e),
                };
                let success = w.client(creds).is_ok();
                //XXX: send more than just a bool, maybe complete err msg
                tx2.send(success)?;
            }
            None => {
                tx2.send(false)?;
                bail!("wifi not initialized")
            }
        }
        Ok(())
    }

    pub(crate) fn new(modem: Modem, store: &DStore) -> anyhow::Result<Self> {
        let mut wifi = Wifi::new(modem, None).expect("Failed to create wifi");
        wifi.ap(wifi::Creds::from_str(CONFIG.wifi_ssid, CONFIG.wifi_psk))
            .expect("Failed to start AP");

        if let Ok(creds) = wifi::Creds::from_store(store) {
            if let Err(e) = wifi.client(creds) {
                println!("Failed to connect to stored wifi: {}", e);
            };
        }

        let server_config = Configuration::default();
        let server = EspHttpServer::new(&server_config)?;

        let conn = Self {
            wifi: Some(wifi),
            server,
        };

        Ok(conn)
    }
}
