use anyhow::{Result, anyhow};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, publisher::Publisher, subscriber::Subscriber},
};

/// Global events which some tasks may want to react to
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Events {
    SetWiFi,
    SetDaemonUrl,
    Start,
}

static EVENTS: PubSubChannel<CriticalSectionRawMutex, Events, 10, 10, 1> = PubSubChannel::new();

pub type EventPublisher = Publisher<'static, CriticalSectionRawMutex, Events, 10, 10, 1>;
pub type EventSubscriberRef<'a> = Subscriber<'a, CriticalSectionRawMutex, Events, 10, 10, 1>;
pub type EventSubscriber = EventSubscriberRef<'static>;

pub fn publisher() -> Result<EventPublisher> {
    // the error here is only publishers or subscribers max reached
    EVENTS.publisher().map_err(|_| anyhow!("max publishers reached in eventbus"))
}

pub fn subscriber() -> Result<EventSubscriber> {
    EVENTS.subscriber().map_err(|_| anyhow!("max subscribers reached in eventbus"))
}
