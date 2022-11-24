
use std::sync::{Mutex, Condvar, Arc};
use anyhow::Result;

use embedded_svc::http::server::{Method};
use embedded_svc::io::Write;


#[allow(unused_variables)]
#[cfg(feature = "experimental")]
pub fn init_httpd(
    mutex: Arc<(Mutex<Option<u32>>, Condvar)>,
) -> Result<esp_idf_svc::http::server::EspHttpServer> {
    let mut server = esp_idf_svc::http::server::EspHttpServer::new(&Default::default())?;

    server
        .fn_handler("/", Method::Get, |req| {
            req.into_ok_response()?
                .write_all("Hello from Rust!".as_bytes())?;

            Ok(())
        })?
        .fn_handler("/foo", Method::Get, |_req| {
            Result::Err("Boo, something happened!".into())
        })?
        .fn_handler("/bar", Method::Get, |req| {
            req.into_response(403, Some("No permissions"), &[])?
                .write_all("You have no permissions to access this page".as_bytes())?;

            Ok(())
        })?
        .fn_handler("/panic", Method::Get, |_req| {
            panic!("User requested a panic!")
        })?;

    Ok(server)
}