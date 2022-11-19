use core::str;
use anyhow::{bail};
// use std::{thread::sleep, time::Duration};

// use bsc::{temp_sensor::BoardTempSensor, wifi::wifi};
use embedded_svc::http::Method;

use esp_idf_hal::modem::Modem;

mod wifi;
use wifi::Wifi;

use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

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
    pub(crate) fn start_server(modem : Modem) -> anyhow::Result<Self> {
        
        let mut wifi = Wifi::new(modem).expect("Failed to create wifi");
        wifi.ap(CONFIG.wifi_ssid, CONFIG.wifi_psk).expect("Failed to start AP");

        let server_config = Configuration::default();
        let mut server = EspHttpServer::new(&server_config)?;
        let x = server
            .fn_handler("/", Method::Get, |request| {
                let html = Self::index_html();
                println!("someone requested the index page");
                request
                    .into_response(
                        200,
                        Some("zueckrali"),//Some(html.as_str()),
                        &[
                            // ("content-type", "text/html"),
                            // ("content-length", format!("{}", html.len()).as_str()),
                        ],
                    )
                    .unwrap().connection().write(html.as_bytes()).unwrap(); //or(anyhow::bail!("Failed to create response"));
                Ok(())
            })?
            .fn_handler("/cool", Method::Get, move |r| {
                r.into_ok_response().unwrap();
                Ok(())
            })?;

        let conn = Connection {
            wifi: Some(wifi),
            server,
        };

        // let mut temp_sensor = BoardTempSensor::new_taking_peripherals();

    
        println!("server awaiting connection");

        // // prevent program from exiting
        // //TO-DO: remove this if possible, or start a new thread with stuff
        // loop {
        //     // let current_temperature = temp_sensor.read_owning_peripherals();
        //     // println!("board temperature: {:.2}", current_temperature);
        //     // println!("");
        //     // x.;
        //     sleep(Duration::from_millis(1000));
        // }
        Ok(conn)
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
        Self::templated("Hello from mcu!")
    }
}
