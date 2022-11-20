use anyhow::bail;
use common::store::DStore;
use core::str;
// use std::borrow::BorrowMut;
use std::sync::mpsc::channel;

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
// use log::info;
use wifi::Wifi;

use esp_idf_svc::http::server::{
    Configuration, EspHttpConnection as SEspHttpConnection, EspHttpServer,
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
    // store: &'a mut DStore,
    // client_credential_channel: (Sender<(String,String)>, Receiver<(String,String)>),
}
const BODY_BUFFER_SIZE: u16 = 1024;
fn parse_multiline(r: &mut SRequest<&mut SEspHttpConnection>) -> anyhow::Result<Vec<String>> {
    let ref mut buf = [0u8; BODY_BUFFER_SIZE as usize];
    let len = r.read(buf)?;
    // println!("{:x?}", buf);
    let body_str = match std::str::from_utf8(&buf[0..len]) {
        Ok(s) => {
            // println!("/connect to: {}", s);
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
    pub(crate) fn start_service(&mut self, store: &mut DStore) -> anyhow::Result<()> {
        let server = &mut self.server;
        server
            .fn_handler("/", Method::Get, |request| {
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
            })
            .unwrap();
        server
            .fn_handler("/status", Method::Get, move |r| {
                r.into_ok_response().unwrap();
                Ok(())
            })
            .unwrap();

        // let wifi: &'a mut Wifi = &mut self.wifi.as_mut().unwrap();

        //FIXME: somehow wrap this in an mpsc so the thread where wifi lives runs this not the thread of the /connect handler
        // let client = |ssid, psk|{
        //     self.wifi.unwrap().client(ssid, psk);
        // };
        let (tx, rx) = channel();
        let (tx2, rx2) = channel();
        server
            .fn_handler("/connect", Method::Post, move |mut r| {
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
                let ssid = data[0].clone();
                let psk = data[1].clone();
                // self.client_credential_channel.0.send((ssid.to_string(), psk.to_string())).unwrap();
                tx.send((ssid, psk)).unwrap();
                // client(ssid, psk);
                match rx2.recv() {
                    Ok(true) => r.into_ok_response().unwrap(),
                    Err(_) | Ok(false) => r.into_status_response(500).unwrap(),
                };
                Ok(())
            })
            .unwrap();

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
            // store,
            // client_credential_channel: channel(),
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
