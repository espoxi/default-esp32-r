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

use esp_idf_hal::modem::Modem;

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
        self.esp_wifi.start()?;
        Ok(())
        // Ok(*self)
    }

    pub fn client(&mut self, ssid: &str, psk: &str) -> anyhow::Result<()> {
        let mut auth_method = AuthMethod::WPAWPA2Personal;
        check_credentials(ssid, psk, &mut auth_method)?;

        info!("Searching for Wifi network {}", ssid);
        let ap_infos = self.esp_wifi.scan()?;
        let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);
        let channel = if let Some(ours) = ours {
            info!(
                "Found configured access point {} on channel {}",
                ssid, ours.channel
            );
            Some(ours.channel)
        } else {
            info!("Configured access point {} not found during scanning, will go with unknown channel",  ssid);
            None
        };

        info!("setting Wifi configuration");

        let client_config = ClientConfiguration {
            ssid: ssid.into(),
            password: psk.into(),
            auth_method,
            channel,
            ..Default::default()
        };
        self.config.client = Some(client_config);

        self.load_cfg()?;
        self.esp_wifi.start()?;

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
        //     info!("Wifi connected");
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
            WConfig{client: None, ap: None} => bail!("No wifi config"),
            WConfig{client: Some(c), ap: None} => wifi::Configuration::Client(c),
            WConfig{client: None, ap: Some(ap)} => wifi::Configuration::AccessPoint(ap),
            WConfig{client: Some(c), ap: Some(ap)} => wifi::Configuration::Mixed(c, ap),
        };
        if let Err(e) = self.esp_wifi.set_configuration(&config) {
            bail!("Error setting wifi config: {:?}", e)
        } else {
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