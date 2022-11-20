use anyhow::bail;

use crate::events::wifi::Creds;

use super::DStore;

pub trait SelfStorable{
    fn new(ssid: String, psk: String) -> Self ;

    fn from_store(store: &DStore) -> anyhow::Result<Self> where Self: Sized;

    fn store_in(&self, store:&mut DStore) -> anyhow::Result<()>;
}


impl SelfStorable for Creds {
    fn new(ssid: String, psk: String) -> Self {
        Self { ssid, psk }
    }

    // fn from_str(ssid: &str, psk: &str) -> Self {
    //     Self {
    //         ssid: ssid.to_string(),
    //         psk: psk.to_string(),
    //     }
    // }

    fn from_store(store: &DStore) -> anyhow::Result<Self>
    {
        match store.get::<Self>("main_creds"){
            Ok(Some(creds)) => Ok(creds),
            Ok(None) => bail!("No credentials found in store"),
            Err(e) => bail!("Failed to get credentials from store: {}", e),
        }
    }

    fn store_in(&self, store:&mut DStore) -> anyhow::Result<()>
    {
        match store.set("main_creds", self){
            Ok(_) => Ok(()),
            Err(e) => bail!("Failed to store credentials in store: {}", e),
        }
    }

}