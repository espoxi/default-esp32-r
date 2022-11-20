use common::events::{Event, ApiEvent};



pub fn handle(e:Event){
    match e {
        Event::Api(ae) => handle_api(ae),
    }
}

fn handle_api(ae:ApiEvent){
    match ae {
        ApiEvent::ConnectToWifi(ssid, psk) => {
            println!("ssid: {}, psk: {}", ssid, psk);
        }
    }
}