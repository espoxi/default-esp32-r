

use std::sync::mpsc::{Sender};

use anyhow::{Result, bail};

use embedded_svc::http::server::{Method, Request};
use embedded_svc::io::{Write, Read};
use esp_idf_svc::http::server::EspHttpConnection;
use log::info;
use serde::de;

use super::wifi::Creds;



#[cfg(feature = "experimental")]
pub fn init_server(

) -> Result<esp_idf_svc::http::server::EspHttpServer> {
    let mut server = esp_idf_svc::http::server::EspHttpServer::new(&Default::default())?;

    server
        .fn_handler("/", Method::Get, |req| {
            req.into_ok_response()?
                .write_all(index_html().as_bytes())?;
            Ok(())
        })?;
    Ok(server)
}

pub fn add_connect_route(server: &mut esp_idf_svc::http::server::EspHttpServer, tx : Sender<super::ConnectionEvent>) -> Result<()> {
    server
        .fn_handler("/connect", Method::Post, move |mut req| {
            let mut buf = Vec::new();
            let creds: Creds = parse_req_json_to(&mut req, &mut buf)?;
            info!("Got creds: {:?}", creds);
            tx.send(super::ConnectionEvent::ConnectToWifi(creds)).unwrap();
            req.into_ok_response()?
                .write_all(index_html().as_bytes())?;
            Ok(())
        })?;
    Ok(())
}
pub fn add_rename_route(server: &mut esp_idf_svc::http::server::EspHttpServer, tx : Sender<super::ConnectionEvent>) -> Result<()> {
    server
        .fn_handler("/rename", Method::Post, move |mut req| {
            let mut buf = Vec::new();
            let creds: Creds = parse_req_json_to(&mut req, &mut buf)?;
            info!("Got creds: {:?}", creds);
            tx.send(super::ConnectionEvent::HostAs(creds)).unwrap();
            req.into_ok_response()?
                .write_all(index_html().as_bytes())?;
            Ok(())
        })?;
    Ok(())
}

// const BODY_BUFFER_SIZE: u16 = 1024;
fn parse_req_json_to<'a, T>(r: &mut Request<&mut EspHttpConnection>, buf: &'a mut [u8]) -> anyhow::Result<T> where
T: de::Deserialize<'a>+Clone,{
    // let mut buf = Vec::new();//[0u8; BODY_BUFFER_SIZE as usize];
    let len = r.read(buf)?;
    let data = match serde_json::from_slice::<T>(&buf[0..len]){
        Ok(t) => t,
        Err(e) => bail!("Failed to parse request body: {}", e),
    };
    Ok(data)
}

#[allow(dead_code)]
fn templated(content: impl AsRef<str>) -> String {templated_with_head(content, "")}
fn templated_with_head(content: impl AsRef<str>, head: impl AsRef<str>,) -> String {
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
    templated_with_head("Please download the app", r#"<meta http-equiv="Refresh" content="0; URL=https://example.com/" />"#)
}