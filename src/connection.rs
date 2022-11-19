use anyhow::bail;
use core::str;
use std::borrow::BorrowMut;
// use std::{thread::sleep, time::Duration};

// use bsc::{temp_sensor::BoardTempSensor, wifi::wifi};
use embedded_svc::{
    http::{
        server::{HandlerError, Request as SRequest},
        Method,
    },
    io::Read,
};

use esp_idf_hal::modem::Modem;

mod wifi;
use log::info;
use wifi::Wifi;

use esp_idf_svc::http::{
    client::EspHttpConnection as CEspHttpConnection,
    server::{Configuration, EspHttpConnection as SEspHttpConnection, EspHttpServer},
};
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
const BODY_BUFFER_SIZE: u16 = 1024;
fn parse_multiline(r: &mut SRequest<&mut SEspHttpConnection>) -> anyhow::Result<Vec<String>> {
    let ref mut buf = [0u8; BODY_BUFFER_SIZE as usize];
    let len = r.read(buf)?;
    // println!("{:x?}", buf);
    let body_str = match std::str::from_utf8(&buf[0..len]) {
        Ok(s) => {
            info!("/connect to: {}", s);
            s
        }
        Err(e) => {
            // r.into_status_response(400).unwrap();
            // return Err(HandlerError::new(
            bail!("Failed to parse body; Invalid UTF-8 sequence: {}", e);
            // ));
        }
    };
    // println!("body:\n {}", body_str);
    let data: Vec<String> = body_str.split("\n").map(str::to_string).collect();
    println!("data: {:?}", data);
    Ok(data)
}

impl<'a> Connection<'a> {
    pub(crate) fn start_server(&mut self) {
        self.server.fn_handler("/", Method::Get, |request| {
            let html = Self::index_html();
            println!("someone requested the index page");
            request
                .into_response(
                    200,
                    Some("zueckrali"), //Some(html.as_str()),
                    &[
                        // ("content-type", "text/html"),
                        // ("content-length", format!("{}", html.len()).as_str()),
                    ],
                )
                .unwrap()
                .connection()
                .write(html.as_bytes())
                .unwrap(); //or(anyhow::bail!("Failed to create response"));
            Ok(())
        });
        self.server.fn_handler("/status", Method::Get, move |r| {
            r.into_ok_response().unwrap();
            Ok(())
        });

        // let wifi: &'a mut Wifi = &mut self.wifi.as_mut().unwrap();

        //FIXME: somehow wrap this in an mpsc so the thread where wifi lives runs this not the thread of the /connect handler
        let client = |ssid, psk|{
            self.wifi.unwrap().client(ssid, psk);
        };
        self.server.fn_handler("/connect", Method::Post, |mut r| {
            let data = match parse_multiline(&mut r) {
                Ok(d) => d,
                Err(e) => {
                    r.into_status_response(400).unwrap();
                    return Err(HandlerError::new(&format!(
                        "Failed to parse body; Invalid UTF-8 sequence: {}",
                        e
                    )));
                }
            };
            let ssid = &data[0];
            let psk = &data[1];
            client(ssid, psk);
            r.into_ok_response().unwrap();
            Ok(())
        });

        println!("server awaiting connection");
    }

    pub(crate) fn new(modem: Modem) -> anyhow::Result<Self> {
        let mut wifi = Wifi::new(modem).expect("Failed to create wifi");
        wifi.ap(CONFIG.wifi_ssid, CONFIG.wifi_psk)
            .expect("Failed to start AP");

        let server_config = Configuration::default();
        let mut server = EspHttpServer::new(&server_config)?;

        let conn = Connection {
            wifi: Some(wifi),
            server,
        };

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
        Self::templated("Please download the app")
    }
}
