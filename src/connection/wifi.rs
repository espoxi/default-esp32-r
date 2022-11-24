use anyhow::bail;
use anyhow::Result;
use embedded_svc::wifi::AuthMethod;
use embedded_svc::wifi::ClientConfiguration;
use log::{info, warn};

use esp_idf_hal::peripheral;
use esp_idf_svc::eventloop::EspSystemEventLoop;

use embedded_svc::wifi::AccessPointConfiguration;
use esp_idf_svc::wifi::WifiWait;
use esp_idf_svc::wifi::{self as w, EspWifi};
use std::time::Duration;

use embedded_svc::wifi::Configuration;
use esp_idf_svc::netif::{EspNetif, EspNetifWait};

use std::net::Ipv4Addr;

use crate::connection::ping;
use crate::store;

// #[allow(dead_code)]
// #[cfg(not(feature = "qemu"))]
// const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
// #[allow(dead_code)]
// #[cfg(not(feature = "qemu"))]
// const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

pub struct Wlan {
    wifi: Box<EspWifi<'static>>,
    event_loop: EspSystemEventLoop,
    config: WConfig,
}

#[derive(Debug, Clone)]
struct WConfig {
    client: Option<ClientConfiguration>,
    ap: Option<AccessPointConfiguration>,
}

impl Wlan {
    pub fn start(
        modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
        sysloop: EspSystemEventLoop,
    ) -> Result<Self> {
        let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

        wifi.start()?;

        info!("Starting wifi...");

        if !WifiWait::new(&sysloop)?
            .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
        {
            bail!("Wifi did not start");
        }

        let mut sself = Self {
            wifi,
            event_loop: sysloop.clone(),
            config: WConfig {
                client: None,
                ap: None,
            },
        };

        match sself.host_as(Creds {
            ssid: CONFIG.wifi_ssid.into(),
            psk: CONFIG.wifi_psk.into(),
        }) {
            Ok(_) => info!("Wifi started as host"),
            Err(e) => warn!("Wifi hosting failed: {}", e),
        };

        // match sself.connect_to(Creds {
        //     ssid: SSID.into(),
        //     psk: PASS.into(),
        // }) {
        //     Ok(_) => info!("Wifi connected to {}", SSID),
        //     Err(e) => warn!("Wifi connecting failed: {}", e),
        // };

        Ok(sself)
    }

    pub fn connect_to(&mut self, creds: Creds) -> Result<()> {
        let (ssid, psk) = (creds.ssid.as_str(), creds.psk.as_str());
        let mut auth_method = AuthMethod::WPAWPA2Personal;
        check_credentials(ssid, psk, &mut auth_method)?;

        info!("Wifi scan");

        let ap_infos = self.wifi.scan()?;

        let ours = &ap_infos.into_iter().find(|a| a.ssid == ssid);

        let channel = if let Some(ours) = ours {
            info!(
                "Found configured access point {} on channel {}",
                ssid, ours.channel
            );
            Some(ours.channel)
        } else {
            info!(
                "Configured access point {} not found during scanning, will go with unknown channel",
                ssid
            );
            None
        };

        let client_config = ClientConfiguration {
            ssid: ssid.into(),
            password: psk.into(),
            auth_method: match ours {
                Some(ref o) => o.auth_method,
                None => auth_method,
            },
            channel,
            ..Default::default()
        };
        self.config.client = Some(client_config);
        self.load_cfg()?;

        self.wifi.start()?;

        info!("Starting wifi...");

        if !WifiWait::new(&self.event_loop)?
            .wait_with_timeout(Duration::from_secs(20), || self.wifi.is_started().unwrap())
        {
            bail!("Wifi did not start");
        }

        info!("Connecting wifi...");

        self.wifi.connect()?;

        if !EspNetifWait::new::<EspNetif>(self.wifi.sta_netif(), &self.event_loop)?
            .wait_with_timeout(Duration::from_secs(20), || {
                self.wifi.is_up().unwrap()
                    && self.wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
            })
        {
            bail!("Wifi did not connect or did not receive a DHCP lease");
        }

        let ip_info = self.wifi.sta_netif().get_ip_info()?;

        info!("Wifi DHCP info: {:?}", ip_info);

        ping(ip_info.subnet.gateway)?;

        Ok(())
    }

    pub fn host_as(&mut self, creds: Creds) -> Result<()> {
        let (ssid, psk) = (creds.ssid.as_str(), creds.psk.as_str());
        let mut auth_method = AuthMethod::WPAWPA2Personal;
        check_credentials(ssid, psk, &mut auth_method)?;

        let ap_config = AccessPointConfiguration {
            ssid: ssid.into(),
            password: psk.into(),
            auth_method: auth_method,
            ..Default::default()
        };

        self.config.ap = Some(ap_config);

        self.load_cfg()?;

        self.wifi.start()?;

        info!("Starting wifi...");

        if !WifiWait::new(&self.event_loop)?
            .wait_with_timeout(Duration::from_secs(20), || self.wifi.is_started().unwrap())
        {
            bail!("Wifi did not start");
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn disable_ap(&mut self) -> anyhow::Result<()> {
        self.config.ap = None;
        self.load_cfg()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn disable_client(&mut self) -> anyhow::Result<()> {
        self.config.client = None;
        self.load_cfg()?;
        Ok(())
    }

    fn load_cfg(&mut self) -> anyhow::Result<()> {
        // use embedded_svc::wifi::Configuration;
        let config = match self.config.clone() {
            WConfig {
                client: None,
                ap: None,
            } => match self.wifi.stop() {
                Ok(()) => return Ok(()),
                Err(e) => bail!("Could not stop Wifi: {:?}", e),
            },
            WConfig {
                client: Some(c),
                ap: None,
            } => Configuration::Client(c),
            WConfig {
                client: None,
                ap: Some(ap),
            } => Configuration::AccessPoint(ap),
            WConfig {
                client: Some(c),
                ap: Some(ap),
            } => Configuration::Mixed(c, ap),
            // wifi::Configuration::Client(c),
        };

        if let Err(e) = self.wifi.set_configuration(&config) {
            bail!("Error setting wifi config: {:?}", e)
        } else {
            println!("Restarting Wifi with new config");
            // self.wifi.start()?;
            Ok(())
        }
    }
}

fn check_credentials(ssid: &str, psk: &str, auth_method: &mut AuthMethod) -> anyhow::Result<()> {
    if ssid.is_empty() {
        bail!("missing WiFi name")
    }
    if psk.is_empty() {
        *auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }
    Ok(())
}

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Creds {
    pub ssid: String,
    pub psk: String,
}

impl store::SelfStorable for Creds {
    fn new(ssid: String, psk: String) -> Self {
        Self { ssid, psk }
    }

    fn from_store(store: &store::DStore) -> anyhow::Result<Self> {
        match store.get::<Self>("main_creds") {
            Ok(Some(creds)) => Ok(creds),
            Ok(None) => bail!("No credentials found in store"),
            Err(e) => bail!("Failed to get credentials from store: {}", e),
        }
    }

    fn store_in(&self, store: &mut store::DStore) -> anyhow::Result<()> {
        match store.set("main_creds", self) {
            Ok(_) => Ok(()),
            Err(e) => bail!("Failed to store credentials in store: {}", e),
        }
    }
}