
pub mod wifi;
use wifi::Creds;


#[macro_export]
macro_rules! api_event {
    ($($t:tt)*) => {
        Event::Api(ApiEvent::$($t)*)
    };
}

pub enum Event{
    Api(ApiEvent),
}

pub enum ApiEvent {
    ConnectToWifi(Creds),
}
