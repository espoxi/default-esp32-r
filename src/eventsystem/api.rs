use std::sync::Arc;

use anyhow::bail;
use common::{events::ApiEvent, store::{storeable::SelfStorable, DStore}};

use crate::connection::Connection;

pub struct ApiEventHandler<'a> {
    conn: Connection<'a>,
    // store: Arc<DStore>,
}

impl<'a> ApiEventHandler<'a> {
    pub fn new(conn: Connection<'a>, /* store : &DStore*/) -> Self {
        Self { conn, 
            // store, 
        }
    }

    pub fn handle(&self, ae: ApiEvent)->anyhow::Result<()> {
        match ae {
            ApiEvent::ConnectToWifi(creds) => {
                println!("ssid: {}, psk: {}", creds.ssid, creds.psk);
                Ok(())
                // match self.conn.wifi {
                //     Some(ref mut w) => {
                //         match creds.store_in(self.store) {
                //             Ok(_) => println!("stored wifi credentials"),
                //             Err(e) => println!("failed to store wifi credentials: {}", e),
                //         };
                //         let success = w.client(creds).is_ok();
                //         //XXX: send more than just a bool, maybe complete err msg
                //         // tx2.send(success)?;
                //         Ok(())
                //     }
                //     None => {
                //         // tx2.send(false)?;
                //         bail!("wifi not initialized")
                //     }
                // }
            }
        }
    }
}
