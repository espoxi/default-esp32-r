use common::events::{Event};
// use esp_idf_hal::delay;
use std::{sync::mpsc::{channel, Receiver, Sender}
// , thread
};
pub mod api;

pub struct EventHandler<'a> {
    pub channel: (Sender<Event>, Receiver<Event>),
    api_handler: api::ApiEventHandler<'a>,
}

pub fn mk_queue() -> (Sender<Event>, Receiver<Event>) {
    channel()
}

impl<'a> EventHandler<'a> {
    pub fn init(
        channel: (Sender<Event>, Receiver<Event>),
        api_handler: api::ApiEventHandler<'a>,
    ) -> Self {
        // let channel = channel();
        Self {
            channel,
            api_handler,
        }
    }

    // pub fn get_tx(&self) -> Sender<Event> {
    //     self.channel.0.clone()
    // }
    // pub fn start_handling(&self)->!{
    //         loop {
    //             let event = self.channel.1.recv().unwrap();
    //             self.handle(event);
    //             delay::FreeRtos::delay_ms(100);
    //         }
    // }

    pub fn handle(&self, e: Event)->anyhow::Result<()> {
        match e {
            Event::Api(ae) => self.api_handler.handle(ae),
        }
    }
}
