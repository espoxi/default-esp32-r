use esp_idf_svc::sntp;

pub trait TimeProvider {
    fn now(&self) -> Option<std::time::Duration>;
    fn new() -> Self;
}

pub struct ESP_NTPC {
    pub sntp: sntp::EspSntp,
    // conf : sntp::SntpConf,
}

impl TimeProvider for ESP_NTPC {
    fn now(&self) -> Option<std::time::Duration> {
        let status = self.sntp.get_sync_status();
        // if status == sntp::SyncStatus::SntpSynced {
        //     let mut tv = esp_idf_sys::timeval {
        //         tv_sec: 0,
        //         tv_usec: 0,
        //     };
        //     unsafe {
        //         sntp_get_timeval(&mut tv);
        //     }
        //     Some(std::time::Duration::from_secs(tv.tv_sec as u64))
        // } else {
        //     None
        // }
        None // TODO
    }
    fn new() -> Self {
        let conf = sntp::SntpConf {
            servers: [
                "2.de.pool.ntp.org",
            ],
            ..Default::default()
        };
        let sntp = sntp::EspSntp::new(&conf).unwrap();
        Self {
            sntp,
        }
    }
}

pub struct ESP_RTC {
   
}

impl TimeProvider for ESP_RTC {
    fn new() -> Self {
        Self {
        }
    }
    fn now(&self) -> Option<std::time::Duration> {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()
    }
}