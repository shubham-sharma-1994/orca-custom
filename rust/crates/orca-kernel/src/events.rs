use crate::manifest::PluginId;
use serde_json::Value;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct Event {
    pub topic: String,
    pub source: PluginId,
    pub payload: Value,
}

#[derive(Clone)]
pub(crate) struct EventBus {
    tx: broadcast::Sender<Event>,
}

impl EventBus {
    pub(crate) fn new() -> Self {
        Self { tx: broadcast::channel(1024).0 }
    }
    pub(crate) fn publish(&self, event: Event) {
        // No subscribers is fine.
        let _ = self.tx.send(event);
    }
    pub(crate) fn subscribe(&self, prefix: &str) -> Subscription {
        Subscription { rx: self.tx.subscribe(), prefix: prefix.to_owned() }
    }
}

/// Receives events whose topic starts with the subscribed prefix.
pub struct Subscription {
    rx: broadcast::Receiver<Event>,
    prefix: String,
}

impl Subscription {
    /// Returns `None` once the kernel shuts down. Lagged events are dropped.
    pub async fn recv(&mut self) -> Option<Event> {
        loop {
            match self.rx.recv().await {
                Ok(e) if e.topic.starts_with(&self.prefix) => return Some(e),
                Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}
