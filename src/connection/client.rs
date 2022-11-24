use log::info;

pub fn test_https_client() -> anyhow::Result<()> {
    use embedded_svc::http::{client::*, };
    use embedded_svc::io::Read;
    use embedded_svc::utils::io;
    use esp_idf_svc::http::client::*;

    let url = String::from("https://google.com");

    info!("About to fetch content from {}", url);

    let mut client = Client::wrap(EspHttpConnection::new(&Configuration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

        ..Default::default()
    })?);

    let mut response = client.get(&url)?.submit()?;

    let mut body = [0_u8; 3048];

    let read = io::try_read_full(&mut response, &mut body).map_err(|err| err.0)?;

    info!(
        "Body (truncated to 3K):\n{:?}",
        String::from_utf8_lossy(&body[..read]).into_owned()
    );

    // Complete the response
    while response.read(&mut body)? > 0 {}

    Ok(())
}