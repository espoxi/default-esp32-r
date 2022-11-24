use embedded_svc::ipv4;
use esp_idf_svc::ping;
use log::info;
use anyhow::{Result, bail};

pub mod wifi;
pub mod server;
pub mod client;

pub fn ping(ip: ipv4::Ipv4Addr) -> Result<()> {
    info!("About to do some pings for {:?}", ip);

    let ping_summary = ping::EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!("Pinging IP {} resulted in timeouts", ip);
    }

    info!("Pinging done");

    Ok(())
}