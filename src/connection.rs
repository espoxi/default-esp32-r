use core::str;
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};


// use bsc::{temp_sensor::BoardTempSensor, wifi::wifi};
use embedded_svc::{
    http::{server::{Response, FnHandler}, Method},
    io::Write,
};

mod wifi;

use esp_idf_svc::{
    http::server::{Configuration, EspHttpServer},
    wifi::EspWifi,
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

pub(crate) fn start_server<'a>() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    // let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);

    let wifi = wifi::wifi(CONFIG.wifi_ssid, CONFIG.wifi_psk)?;

    // let mut temp_sensor = BoardTempSensor::new_taking_peripherals();

    let server_config = Configuration::default();
    let mut server = EspHttpServer::new(&server_config)?;
    server.handler("/",Method::Get ,FnHandler::new(|request| {
        let html = index_html();
        request.into_response(200,Some(html.as_str()) , &[]);
        Ok(())
    }))?;

    println!("server awaiting connection");

    // // prevent program from exiting
    // loop {
    //     // let current_temperature = temp_sensor.read_owning_peripherals();
    //     // println!("board temperature: {:.2}", current_temperature);
    //     sleep(Duration::from_millis(1000));
    // }
    Ok(())
}

fn templated(content: impl AsRef<str>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>esp-rs web server</title>
    </head>
    <body>
        {}
    </body>
</html>
"#,
        content.as_ref()
    )
}

fn index_html() -> String {
    templated("Hello from mcu!")
}
