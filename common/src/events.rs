pub enum Event{
    Api(ApiEvent),
}

pub enum ApiEvent {
    ConnectToWifi(String, String),
}