use std::sync::Arc;

use anyhow::bail;
use common::{
    events::ApiEvent,
    store::{storeable::SelfStorable, DStore},
};

use crate::connection::Connection;

pub struct ApiEventHandler<'a, 'b> {
    conn: &'a mut Connection<'b>,
    store: &'a mut DStore,
}

impl<'a, 'b> ApiEventHandler<'a, 'b> {
    pub fn new(conn: &'a mut Connection<'b>, store: &'a mut DStore) -> Self {
        Self { conn, store }
    }

    pub fn handle(&mut self, ae: ApiEvent) -> anyhow::Result<()> {
        match ae {
            ApiEvent::ConnectToWifi(creds) => {
                // println!("ssid: {}, psk: {}", creds.ssid, creds.psk);
                // Ok(())
                match self.conn.wifi {
                    Some(ref mut w) => {
                        match creds.store_in(self.store) {
                            Ok(_) => println!("stored wifi credentials"),
                            Err(e) => println!("failed to store wifi credentials: {}", e),
                        };
                        match w.client(creds) {
                            Ok(()) => Ok(()),
                            Err(e) => {
                                println!("failed to connect t wifi: {}", e);
                                Err(e)
                            }
                        }
                        //XXX: send more than just a bool, maybe complete err msg
                        // tx2.send(success)?;
                    }
                    None => {
                        // tx2.send(false)?;
                        bail!("wifi not initialized")
                    }
                }
            }
        }
    }
}
