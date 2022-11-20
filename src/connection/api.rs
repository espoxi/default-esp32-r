use std::sync::mpsc::Sender;

use anyhow::bail;
use common::{events::{Event,ApiEvent, wifi::Creds}, api_event};
use embedded_svc::{http::{server::{HandlerError, Request as SRequest}, Method}, io::Read};
use esp_idf_svc::http::server::{EspHttpConnection as SEspHttpConnection, EspHttpServer};

macro_rules! handler_bail {
    ($($t:tt)*) => {
        return Err(HandlerError::new(format!($($t)*).as_str()))
    };
}

pub fn init(server: &mut EspHttpServer, tx: Sender<Event>) {
    server
        .fn_handler("/", Method::Get, |request| {
            let html = index_html();
            match request.into_response(
                200,
                Some("zueckrali"), //Some(html.as_str()),
                &[],
            ) {
                Ok(mut r) => match r.connection().write(html.as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(e) => handler_bail!("Failed to write response: {}", e),
                },
                Err(e) => handler_bail!("Failed to get response: {}", e),
            }
        })
        .unwrap();
    server
        .fn_handler("/status", Method::Get, move |r| {
            r.into_ok_response().unwrap();
            Ok(())
        })
        .unwrap();

    server
        .fn_handler("/connect", Method::Post, move |mut r| {
            let data = match parse_multiline(&mut r) {
                Ok(d) => d,
                Err(e) => {
                    r.into_status_response(400).unwrap();
                    handler_bail!("Failed to parse body; Invalid UTF-8 sequence: {}", e);
                }
            };
            let ssid = data[0].clone();
            let psk = data[1].clone();

            tx.send(api_event!(ConnectToWifi(Creds{ssid,psk}))).unwrap();
            // client(ssid, psk);
            // match rx2.recv() {
            //     Ok(true) => r.into_ok_response().unwrap(),
            //     Err(_) | Ok(false) => r.into_status_response(500).unwrap(),
            // };
            r.into_ok_response().unwrap();
            Ok(())
        })
        .unwrap();
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
    templated("Please download the app")
}

const BODY_BUFFER_SIZE: u16 = 1024;
fn parse_multiline(r: &mut SRequest<&mut SEspHttpConnection>) -> anyhow::Result<Vec<String>> {
    let ref mut buf = [0u8; BODY_BUFFER_SIZE as usize];
    let len = r.read(buf)?;
    let body_str = match std::str::from_utf8(&buf[0..len]) {
        Ok(s) => s,
        Err(e) => {
            bail!("Failed to parse body; Invalid UTF-8 sequence: {}", e)
        }
    };
    let data: Vec<String> = body_str.split("\n").map(str::to_string).collect();
    println!("data: {:?}", data);
    Ok(data)
}
