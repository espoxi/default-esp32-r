use esp_idf_svc::{sntp, systime};

pub trait TimeProvider {
    fn now(&self) -> Option<std::time::Duration>;
    fn clone(&self) -> Box<dyn TimeProvider + Send>;
}

pub struct ESP_NTPC {
    pub sntp: sntp::EspSntp,
    // conf : sntp::SntpConf,
    timer: systime::EspSystemTime,
}

impl ESP_NTPC {
    pub fn new() -> Self {
        let conf = sntp::SntpConf {
            servers: ["2.de.pool.ntp.org"],
            ..Default::default()
        };
        let sntp = sntp::EspSntp::new(&conf).unwrap();
        let timer = systime::EspSystemTime;
        Self { sntp, timer }
    }
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
        // if status != sntp::SyncStatus::Completed {
        Some(self.timer.now())
        // } else {
        //     None
        // }
    }

    fn clone(&self) -> Box<dyn TimeProvider + Send> {
        Box::new(EspSystemTime {})
    } //FIXME: wrong provider
}

pub struct EspSystemTime {}

impl TimeProvider for EspSystemTime {
    fn now(&self) -> Option<std::time::Duration> {
        Some(systime::EspSystemTime.now())
    }

    fn clone(&self) -> Box<dyn TimeProvider + Send> {
        Box::new(EspSystemTime {})
    }
}
