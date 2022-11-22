// use std::sync::Arc;
use anyhow::bail;
use common::{events::wifi::Creds, store::DStore};
use embedded_svc::wifi::{
    self,
    AccessPointConfiguration,
    AuthMethod,
    ClientConfiguration, Configuration,
    // Wifi as _,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    netif::{EspNetif, EspNetifWait, NetifConfiguration, NetifStack},
    nvs::EspDefaultNvsPartition,
    wifi::{EspWifi, WifiDriver, WifiWait},
};
use log::info;
use std::{net::Ipv4Addr, result::Result::Ok, time::Duration};

use esp_idf_hal::{delay::FreeRtos, modem::Modem, peripheral};
use serde::{Deserialize, Serialize};

use crate::connection::ping;

// use log::info;

pub fn testwifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    SSID: &str,
    PASS: &str,
) -> anyhow::Result<Box<EspWifi<'static>>> {
    use std::net::Ipv4Addr;

    use esp_idf_svc::handle::RawHandle;

    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

    println!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        println!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        println!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    wifi.start()?;

    println!("Starting wifi...");

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    println!("Connecting wifi...");

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            // wifi.is_connected().unwrap() && 
            wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    println!("Wifi DHCP info: {:?}", ip_info);

    // ping(ip_info.subnet.gateway)?;

    Ok(wifi)
}

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    // #[default("")]
    // wifi_psk: &'static str,
}

#[derive(Clone, Debug)]
struct WConfig {
    client: Option<ClientConfiguration>,
    ap: Option<AccessPointConfiguration>,
}

pub struct Wifi<'a> {
    esp_wifi: EspWifi<'a>,
    config: WConfig,
    sysloop: EspSystemEventLoop,
}

impl<'a> Wifi<'a> {
    pub fn new(
        modem: Modem,
        nvsp: Option<EspDefaultNvsPartition>,
        sysloop: EspSystemEventLoop,
    ) -> anyhow::Result<Self> {
        let esp_wifi = EspWifi::new(
            modem,
            sysloop.clone(), //_or_else(return Err(anyhow::anyhow!("No system event loop"))),
            EspDefaultNvsPartition::take().ok(),
        )?;

        // //FIXME: it is still displayed as default "espressif", not CONFIG.wifi_ssid
        // let ipv4_client_cfg =
        //     embedded_svc::ipv4::ClientConfiguration::DHCP(embedded_svc::ipv4::DHCPClientSettings {
        //         hostname: Some(heapless::String::<30>::from(CONFIG.wifi_ssid)),
        //         ..Default::default()
        //     });
        // let new_c = NetifConfiguration {
        //     ip_configuration: embedded_svc::ipv4::Configuration::Client(ipv4_client_cfg),
        //     ..NetifConfiguration::wifi_default_client()
        // };

        // let esp_wifi = EspWifi::wrap_all(
        //     WifiDriver::new(
        //         modem,
        //         EspSystemEventLoop::take().unwrap(), //XXX: if i need to use the sysloop i need to pass it here
        //         nvsp,
        //     )?,
        //     EspNetif::new_with_conf(&new_c)?,
        //     EspNetif::new(NetifStack::Ap)?,
        // )?;

        Ok(Self {
            esp_wifi,
            config: WConfig {
                client: None,
                ap: None,
            },
            sysloop: sysloop.clone(),
        })
    }
    #[allow(dead_code)]
    pub fn disable_ap(&mut self) -> anyhow::Result<()> {
        self.config.ap = None;
        self.load_cfg()?;
        Ok(())
    }
    pub fn ap(&mut self, creds: Creds) -> anyhow::Result<()> {
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
        Ok(())
        // Ok(*self)
    }

    #[allow(dead_code)]
    pub fn disable_client(&mut self) -> anyhow::Result<()> {
        self.config.client = None;
        self.load_cfg()?;
        Ok(())
    }
    pub fn client(&mut self, creds: Creds) -> anyhow::Result<()> {
        let (ssid, psk) = (creds.ssid.as_str(), creds.psk.as_str());
        let mut auth_method = AuthMethod::WPAWPA2Personal;
        check_credentials(ssid, psk, &mut auth_method)?;

        println!("Searching for Wifi network {}", ssid);
        let ap_infos = self.esp_wifi.scan()?;
        let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);
        let channel = if let Some(ours) = &ours {
            println!(
                "Found configured access point {} on channel {}, strength:{}\n we will try to cennect via {}",
                ssid, ours.channel, ours.signal_strength, ours.auth_method
            );
            Some(ours.channel)
        } else {
            println!("Configured access point {} not found during scanning, will go with unknown channel",  ssid);
            None
        };

        println!("setting Wifi configuration");

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

        self.esp_wifi
            .sta_netif_mut()
            .set_mac(&[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])?;

        if !WifiWait::new(&self.sysloop)?
            .wait_with_timeout(Duration::from_secs(20), || self.esp_wifi.is_started().unwrap())
        {
            bail!("Wifi did not start");
        }

        self.esp_wifi.connect()?;

        println!("netif wait");

        if !EspNetifWait::new::<EspNetif>(self.esp_wifi.sta_netif(), &self.sysloop)?
            .wait_with_timeout(Duration::from_secs(20), || {
                self.esp_wifi.driver().is_connected().unwrap()
                    && self.esp_wifi.sta_netif().get_ip_info().unwrap().ip
                        != Ipv4Addr::new(0, 0, 0, 0)
            })
        {
            bail!("Wifi did not connect or did not receive a DHCP lease");
        }

        let d = self.esp_wifi.driver_mut();
        if d.is_sta_enabled().unwrap() {
            println!("activated client, connecting");
            while !d.is_sta_connected().unwrap_or(false) {
                match d.connect() {
                    Ok(()) => {
                        // d.status;
                        break;
                    }
                    Err(_e) => {
                        // println!("Error connecting: {}", e);
                        print!(".");
                        FreeRtos::delay_ms(500);
                    }
                };
            }
            println!(
                "connected to {} as {} ({})",
                ssid,
                self.esp_wifi
                    .sta_netif_mut()
                    .get_hostname()
                    .unwrap_or(heapless::String::<30>::from("unknown"))
                    .as_str(),
                self.esp_wifi.sta_netif_mut().get_ip_info().unwrap().ip,
            );
        }

        let ip_info = self.esp_wifi.sta_netif().get_ip_info()?;

        println!("Wifi DHCP info: {:?}", ip_info);

        ping(ip_info.subnet.gateway)?;

        Ok(())
        // Ok(self)
    }

    fn load_cfg(&mut self) -> anyhow::Result<()> {
        let config = match self.config.clone() {
            WConfig {
                client: None,
                ap: None,
            } => match self.esp_wifi.stop() {
                Ok(()) => return Ok(()),
                Err(e) => bail!("Could not stop Wifi: {:?}", e),
            },
            WConfig {
                client: Some(c),
                ap: None,
            } => wifi::Configuration::Client(c),
            WConfig {
                client: None,
                ap: Some(ap),
            } => wifi::Configuration::AccessPoint(ap),
            WConfig {
                client: Some(c),
                ap: Some(ap),
            } => wifi::Configuration::Mixed(c, ap),
            // wifi::Configuration::Client(c),
        };

        if let Err(e) = self.esp_wifi.set_configuration(&config) {
            bail!("Error setting wifi config: {:?}", e)
        } else {
            println!("Restarting Wifi withz new config");
            self.esp_wifi.start()?;
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
        println!("Wifi password is empty");
    }
    Ok(())
}
