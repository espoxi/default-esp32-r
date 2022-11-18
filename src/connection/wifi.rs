// use std::sync::Arc;

use anyhow::bail;
use embedded_svc::wifi::{
    self,
    AuthMethod,
    // ClientConfiguration, //ClientConnectionStatus, ClientIpStatus, ClientStatus,
    Wifi as _, 
    AccessPointConfiguration,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};

use esp_idf_hal::{
    // peripherals::Peripherals, 
    modem::Modem,
};

use log::info;
// use std::time::Duration;

#[allow(unused)]
pub struct Wifi<'a> {
    esp_wifi: EspWifi<'a>,
}

pub fn wifi<'a>(ssid: &str, psk: &str, modem: Modem) -> anyhow::Result<Wifi<'a>> {
    let mut auth_method = AuthMethod::WPAWPA2Personal; // Todo: add this setting - router dependent
    if ssid.is_empty() {
        bail!("missing WiFi name")
    }
    if psk.is_empty() {
        auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }
    // let netif_stack = Arc::new(EspNetif::new()?);
    // let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    // let default_nvs = Arc::new(EspDefaultNvs::new(
    //     EspDefaultNvsPartition::take().unwrap(),
    //     "defaultns",
    //     true,
    // )?);

    let mut wifi = EspWifi::new(
        // match Peripherals::take(){
        //     Some(p) => p,
        //     None => bail!("Failed to take peripherals"),
        // }.
        modem,
        EspSystemEventLoop::take().unwrap(),//_or_else(return Err(anyhow::anyhow!("No system event loop"))),
        EspDefaultNvsPartition::take().ok(),
    )?;

    // info!("Searching for Wifi network {}", ssid);

    // let ap_infos = wifi.scan()?;

    // let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    // let channel = if let Some(ours) = ours {
    //     info!(
    //         "Found configured access point {} on channel {}",
    //         ssid, ours.channel
    //     );
    //     Some(ours.channel)
    // } else {
    //     info!(
    //         "Configured access point {} not found during scanning, will go with unknown channel",
    //         ssid
    //     );
    //     None
    // };

    // info!("setting Wifi configuration");
    // wifi.set_configuration(&wifi::Configuration::Client(ClientConfiguration {
    //     ssid: ssid.into(),
    //     password: psk.into(),
    //     channel,
    //     auth_method,
    //     ..Default::default()
    // }))?;

    wifi.set_configuration(&wifi::Configuration::AccessPoint(AccessPointConfiguration {
        ssid: ssid.into(),
        password: psk.into(),
        auth_method: auth_method,

        ..Default::default()
    }))?;

    info!("getting Wifi status");

    wifi.start()?;

    info!("Wifi started");


    // wifi.wait_status_with_timeout(Duration::from_secs(2100), |status| {
    //     !status.is_transitional()
    // })
    // .map_err(|err| anyhow::anyhow!("Unexpected Wifi status (Transitional state): {:?}", err))?;

    // let status = wifi.get_status();

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

    let wifi = Wifi { esp_wifi: wifi };

    Ok(wifi)
}
