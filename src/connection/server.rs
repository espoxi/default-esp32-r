use std::sync::mpsc::Sender;

use anyhow::{bail, Result};

use embedded_svc::http::server::{HandlerError, Method, Request};
use embedded_svc::io::{Read, Write};
use esp_idf_svc::http::server::EspHttpConnection;
use log::{info, warn};
use serde::de;

#[allow(unused_imports)]
use super::wifi::Creds;

#[macro_export]
macro_rules! handler_bail {
    ($($t:tt)*) => {
        return Err(embedded_svc::http::server::HandlerError::new(format!($($t)*).as_str()))
    };
}


#[macro_export]
macro_rules! send_as_json {
    ($req:ident, $e:expr) => {
        match serde_json::to_vec(&$e){
            Ok(ref b) => {$req.into_ok_response()?.write_all(b)?;Ok(())},
            Err(e) => {$req.into_status_response(500)?.write_all(e.to_string().as_bytes())?;handler_bail!("whoppa: {:?}", e)},
        }
    };
}

#[macro_export]
macro_rules! match_parsed_json {
    ($req:expr, $($t:tt)*) => {
        {
            let buf = &mut [0u8; crate::connection::server::BODY_BUFFER_SIZE as usize];
            match crate::connection::server::parse_req_json_to(&mut $req, buf)$($t)*
        }
    };
}

#[macro_export]
macro_rules! parse_req_or_fail_with_message {
    ($req:expr; $($t:tt)*) => {
        match_parsed_json!($req,{
            Ok(parsed) => {
                info!("parsed body: {:?}", parsed);
                parsed
            }
            Err(err) => {
                println!("Failed to parse body: {}", err);
                let info = format!($($t)*, err);
                $req.into_status_response(400)?.write_all(info.as_bytes())?;
                handler_bail!("{}", info)
            }
        })
    };
}

pub const BODY_BUFFER_SIZE: u16 = 1024;
// const BODY_BUFFER_SIZE: u16 = 1024;
pub fn parse_req_json_to<'a, T>(
    r: &mut Request<&mut EspHttpConnection>,
    buf: &'a mut [u8],
) -> anyhow::Result<T>
where
    T: de::Deserialize<'a> + Clone,
{
    let len = r.read(buf)?;
    info!("parsing body...\n{}", show(&buf[0..len]));
    match serde_json::from_slice::<T>(&buf[0..len]) {
        Ok(t) => Ok(t),
        Err(e) => {warn!("failed: {}",e);Err(anyhow::anyhow!("Failed to parse request body: {}", e))},
    }
}
use std::ascii::escape_default;
use std::str;

fn show(bs: &[u8]) -> String {
    let mut visible = String::new();
    for &b in bs {
        let part: Vec<u8> = escape_default(b).collect();
        visible.push_str(str::from_utf8(&part).unwrap());
    }
    visible
}

pub fn init_server() -> Result<esp_idf_svc::http::server::EspHttpServer> {
    let mut server = esp_idf_svc::http::server::EspHttpServer::new(&Default::default())?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?.write_all(index_html().as_bytes())?;
        Ok(())
    })?;
    Ok(server)
}

use super::{
    ConnectionEvent::{ConnectToWifi, HostAs},
    ConnectionRelevantEvent::Wifi as WE,
};

macro_rules! send_creds_on_route_as_event {
    ($server:ident, $tx:ident, $url:expr, $event:expr) => {
        add_new_route(
            $server,
            RouteData::new($url, Method::Post, move |mut req| {
                // let creds:Creds = parse_req_or_fail_with_message!(req;"failed parsing creds: {}");
                let buf = &mut [0u8; crate::connection::server::BODY_BUFFER_SIZE as usize];
                let creds = match crate::connection::server::parse_req_json_to(&mut req, buf){
                    Ok(parsed) => {
                        info!("parsed body: {:?}", parsed);
                        parsed
                    }
                    Err(err) => {
                        println!("Failed to parse body: {}", err);
                        let info = format!("failed parsing creds: {}", err);
                        req.into_status_response(400)?.write_all(info.as_bytes())?;
                        handler_bail!("{}", info)
                    }
                };
                $tx.send(WE($event(creds))).unwrap();
                req.into_ok_response().unwrap();
                Ok(())
            }),
        )
    };
}

pub(super) fn add_connect_route(
    server: &mut esp_idf_svc::http::server::EspHttpServer,
    tx: Sender<super::ConnectionRelevantEvent>,
) -> Result<()> {
    send_creds_on_route_as_event!(server, tx, "/connect", ConnectToWifi)
}
pub(super) fn add_rename_route(
    server: &mut esp_idf_svc::http::server::EspHttpServer,
    tx: Sender<super::ConnectionRelevantEvent>,
) -> Result<()> {
    send_creds_on_route_as_event!(server, tx, "/rename", HostAs)
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

pub(crate) fn add_new_route(
    server: &mut esp_idf_svc::http::server::EspHttpServer,
    route_data: RouteData,
) -> anyhow::Result<()> {
    let RouteData {
        uri,
        method,
        handler,
    } = route_data;
    match server.fn_handler(uri.as_str(), method, handler) {
        Ok(_) => Ok(()),
        Err(e) => bail!("Failed to add route: {}", e),
    }
}
