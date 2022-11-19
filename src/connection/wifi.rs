// use std::sync::Arc;

use anyhow::{bail, Ok};
use embedded_svc::wifi::{
    self,
    AccessPointConfiguration,
    AuthMethod,
    ClientConfiguration,
    // Configuration,
    // ClientConfiguration, //ClientConnectionStatus, ClientIpStatus, ClientStatus,
    // Wifi as _,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};

use esp_idf_hal::{delay::FreeRtos, modem::Modem};

use log::info;

#[derive(Clone, Debug)]
struct WConfig {
    client: Option<ClientConfiguration>,
    ap: Option<AccessPointConfiguration>,
}

pub struct Wifi<'a> {
    esp_wifi: EspWifi<'a>,
    config: WConfig,
}

impl<'a> Wifi<'a> {
    pub fn new(modem: Modem) -> anyhow::Result<Self> {
        //let mut
        let esp_wifi = EspWifi::new(
            modem,
            EspSystemEventLoop::take().unwrap(), //_or_else(return Err(anyhow::anyhow!("No system event loop"))),
            EspDefaultNvsPartition::take().ok(),
        )?;
        Ok(Self {
            esp_wifi,
            config: WConfig {
                client: None,
                ap: None,
            },
        })
    }
    pub fn disable_ap(&mut self) -> anyhow::Result<()> {
        self.config.ap = None;
        self.load_cfg()?;
        Ok(())
    }
    pub fn ap(&mut self, ssid: &str, psk: &str) -> anyhow::Result<()> {
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
    pub fn disable_client(&mut self) -> anyhow::Result<()> {
        self.config.client = None;
        self.load_cfg()?;
        Ok(())
    }
    pub fn client(&mut self, ssid: &str, psk: &str) -> anyhow::Result<()> {
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

        self.esp_wifi.connect()?;

        let d = self.esp_wifi.driver_mut();
        if d.is_sta_enabled().unwrap() {
            println!("activated client, connecting");
            while !d.is_sta_connected().unwrap_or(false) {
                match d.connect() {
                    std::result::Result::Ok(()) => {
                        break;
                    }
                    Err(_e) => {
                        // println!("Error connecting: {}", e);
                        print!(".");
                        FreeRtos::delay_ms(500);
                    }
                };
            }
            println!("connected to {}", ssid);
        }

        // self.esp_wifi.wait_status_with_timeout(Duration::from_secs(2100), |status| {
        //     !status.is_transitional()
        // })
        // .map_err(|err| anyhow::anyhow!("Unexpected Wifi status (Transitional state): {:?}", err))?;

        // let status = self.esp_wifi.status();

        // if let wifi::Status(
        //     ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(
        //         _ip_settings,
        //     ))),
        //     _,
        // ) = status
        // {
        //     println!("Wifi connected");
        // } else {
        //     bail!(
        //         "Could not connect to Wifi - Unexpected Wifi status: {:?}",
        //         status
        //     );
        // }

        Ok(())
        // Ok(self)
    }

    fn load_cfg(&mut self) -> anyhow::Result<()> {
        let config = match self.config.clone() {
            WConfig {
                client: None,
                ap: None,
            } => match self.esp_wifi.stop() {
                std::result::Result::Ok(()) => return Ok(()),
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
