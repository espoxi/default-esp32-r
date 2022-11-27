use std::sync::mpsc::Sender;

use anyhow::{bail, Result};

use embedded_svc::http::server::{HandlerError, Method, Request};
use embedded_svc::io::{Read, Write};
use esp_idf_svc::http::server::EspHttpConnection;
use log::info;
use serde::de;

#[allow(unused_imports)]
use super::wifi::Creds;

macro_rules! handler_bail {
    ($($t:tt)*) => {
        return Err(HandlerError::new(format!($($t)*).as_str()))
    };
}

#[cfg(feature = "experimental")]
pub fn init_server() -> Result<esp_idf_svc::http::server::EspHttpServer> {
    let mut server = esp_idf_svc::http::server::EspHttpServer::new(&Default::default())?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?.write_all(index_html().as_bytes())?;
        Ok(())
    })?;
    Ok(server)
}

use super::{ConnectionEvent::{ConnectToWifi,HostAs},ConnectionRelevantEvent::Wifi as WE};

const BODY_BUFFER_SIZE: u16 = 1024;
pub(super) fn add_connect_route(
    server: &mut esp_idf_svc::http::server::EspHttpServer,
    tx: Sender<super::ConnectionRelevantEvent>,
) -> Result<()> {
    add_new_route(server, RouteData::new("/connect", Method::Post, move |mut req| {
        let buf = &mut [0u8; BODY_BUFFER_SIZE as usize];
        match parse_req_json_to(&mut req, buf) {
            Ok(creds) => {
                info!("Got creds: {:?}", creds);
                tx.send(WE(ConnectToWifi(creds)))
                    .unwrap();
                req.into_ok_response().unwrap();
                Ok(())
            }
            Err(e) => {
                let info = format!("failed to parse creds: {}", e);
                req.into_status_response(400)?.write_all(info.as_bytes())?;
                handler_bail!("{}", info)
            }
        }
    }))?;
    Ok(())
}
pub(super) fn add_rename_route(
    server: &mut esp_idf_svc::http::server::EspHttpServer,
    tx: Sender<super::ConnectionRelevantEvent>,
) -> Result<()> {
    add_new_route(server, RouteData::new("/rename", Method::Post, move |mut req| {
        let buf = &mut [0u8; BODY_BUFFER_SIZE as usize];
        match parse_req_json_to(&mut req, buf) {
            Ok(creds) => {
                info!("Got creds: {:?}", creds);
                tx.send(WE(HostAs(creds))).unwrap();
                req.into_ok_response().unwrap();
                Ok(())
            }
            Err(e) => {
                let info = format!("failed to parse creds: {}", e);
                req.into_status_response(400)?.write_all(info.as_bytes())?;
                handler_bail!("{}", info)
            }
        }
    }))?;
    Ok(())
}

// const BODY_BUFFER_SIZE: u16 = 1024;
fn parse_req_json_to<'a, T>(
    r: &mut Request<&mut EspHttpConnection>,
    buf: &'a mut [u8],
) -> anyhow::Result<T>
where
    T: de::Deserialize<'a> + Clone,
{
    // let mut buf = Vec::new();//[0u8; BODY_BUFFER_SIZE as usize];
    let len = r.read(buf)?;
    let data = match serde_json::from_slice::<T>(&buf[0..len]) {
        Ok(t) => t,
        Err(e) => bail!("Failed to parse request body: {}", e),
    };
    Ok(data)
}

#[allow(dead_code)]
fn templated(content: impl AsRef<str>) -> String {
    templated_with_head(content, "")
}
fn templated_with_head(content: impl AsRef<str>, head: impl AsRef<str>) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8">
                {}
                <title>esp-rs web server</title>
            </head>
            <body>
                {}
            </body>
        </html>
        "#,
        head.as_ref(),
        content.as_ref(),
    )
}

fn index_html() -> String {
    templated_with_head(
        "Please download the app",
        r#"<meta http-equiv="Refresh" content="0; URL=https://espoxi.github.io/" />"#,
    )
}

pub struct RouteData {
    uri: String,
    method: Method,
    handler: Box<dyn Fn(Request<&mut EspHttpConnection>) -> Result<(), HandlerError> + Send>,
}
impl RouteData {
    pub fn new(
        uri: impl Into<String>,
        method: Method,
        handler: impl Fn(Request<&mut EspHttpConnection>) -> Result<(), HandlerError> + Send + 'static,
    ) -> Self {
        Self {
            uri: uri.into(),
            method,
            handler: Box::new(handler),
        }
    }
}

pub(crate) fn add_new_route(server: &mut esp_idf_svc::http::server::EspHttpServer, route_data: RouteData) -> anyhow::Result<()> {
    let RouteData { uri, method, handler } = route_data;
    match server.fn_handler(uri.as_str(), method, handler){
        Ok(_) => Ok(()),
        Err(e) => bail!("Failed to add route: {}", e)
    }
}
