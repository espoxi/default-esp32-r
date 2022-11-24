use std::{net::{TcpStream, TcpListener}, thread, io::{Write, Read}};

use anyhow::Result;
use log::{info, error};



pub fn test_tcp() -> Result<()> {
    info!("About to open a TCP connection to 1.1.1.1 port 80");

    let mut stream = TcpStream::connect("one.one.one.one:80")?;

    let err = stream.try_clone();
    if let Err(err) = err {
        info!(
            "Duplication of file descriptors does not work (yet) on the ESP-IDF, as expected: {}",
            err
        );
    }

    stream.write_all("GET / HTTP/1.0\n\n".as_bytes())?;

    let mut result = Vec::new();

    stream.read_to_end(&mut result)?;

    info!(
        "1.1.1.1 returned:\n=================\n{}\n=================\nSince it returned something, all is OK",
        std::str::from_utf8(&result)?);

    Ok(())
}

pub fn test_tcp_bind() -> Result<()> {
    fn test_tcp_bind_accept() -> Result<()> {
        info!("About to bind a simple echo service to port 8080");

        let listener = TcpListener::bind("0.0.0.0:8080")?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!("Accepted client");

                    thread::spawn(move || {
                        test_tcp_bind_handle_client(stream);
                    });
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }

        unreachable!()
    }

    fn test_tcp_bind_handle_client(mut stream: TcpStream) {
        // read 20 bytes at a time from stream echoing back to stream
        loop {
            let mut read = [0; 128];

            match stream.read(&mut read) {
                Ok(n) => {
                    if n == 0 {
                        // connection was closed
                        break;
                    }
                    stream.write_all(&read[0..n]).unwrap();
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }

    thread::spawn(|| test_tcp_bind_accept().unwrap());

    Ok(())
}


// #[cfg(not(esp_idf_version = "4.3"))]
// pub fn test_tcp_bind_async() -> anyhow::Result<()> {
//     async fn test_tcp_bind() -> smol::io::Result<()> {
//         /// Echoes messages from the client back to it.
//         async fn echo(stream: smol::Async<TcpStream>) -> smol::io::Result<()> {
//             smol::io::copy(&stream, &mut &stream).await?;
//             Ok(())
//         }

//         // Create a listener.
//         let listener = smol::Async::<TcpListener>::bind(([0, 0, 0, 0], 8081))?;

//         // Accept clients in a loop.
//         loop {
//             let (stream, peer_addr) = listener.accept().await?;
//             info!("Accepted client: {}", peer_addr);

//             // Spawn a task that echoes messages from the client back to it.
//             smol::spawn(echo(stream)).detach();
//         }
//     }

//     info!("About to bind a simple echo service to port 8081 using async (smol-rs)!");

//     #[allow(clippy::needless_update)]
//     {
//         esp_idf_sys::esp!(unsafe {
//             esp_idf_sys::esp_vfs_eventfd_register(&esp_idf_sys::esp_vfs_eventfd_config_t {
//                 max_fds: 5,
//                 ..Default::default()
//             })
//         })?;
//     }

//     thread::Builder::new().stack_size(4096).spawn(move || {
//         smol::block_on(test_tcp_bind()).unwrap();
//     })?;

//     Ok(())
// }