use crate::commands::{CommandFuture, CommandRegistry};
use crate::error::{BoxError, KernelError, Result};
use crate::events::{Event, EventBus, Subscription};
use crate::manifest::{Manifest, PluginId};
use crate::services::ServiceRegistry;
use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;
use std::sync::Arc;

/// The only extension point. Built-in features and third-party code implement this identically.
#[async_trait]
pub trait Plugin: Send + Sync {
    fn manifest(&self) -> Manifest;
    async fn start(&self, cx: &Context) -> Result<(), BoxError>;
    async fn stop(&self, _cx: &Context) -> Result<(), BoxError> {
        Ok(())
    }
}

pub(crate) struct Shared {
    pub services: ServiceRegistry,
    pub commands: CommandRegistry,
    pub events: EventBus,
}

/// A plugin's view of the kernel, scoped to that plugin's identity and declared dependencies.
#[derive(Clone)]
pub struct Context {
    id: PluginId,
    requires: Arc<[PluginId]>,
    shared: Arc<Shared>,
}

impl Context {
    pub(crate) fn new(manifest: &Manifest, shared: Arc<Shared>) -> Self {
        Self { id: manifest.id.clone(), requires: manifest.requires.clone().into(), shared }
    }

    pub fn plugin_id(&self) -> &PluginId {
        &self.id
    }

    pub fn provide_service<T: Any + Send + Sync>(&self, service: Arc<T>) {
        self.shared.services.provide(&self.id, service);
    }

    /// Resolves a service owned by this plugin or one of its declared `requires`.
    pub fn service<T: Any + Send + Sync>(&self) -> Result<Arc<T>> {
        self.shared.services.get(&self.id, &self.requires)
    }

    pub fn publish(&self, topic: &str, payload: Value) {
        self.shared.events.publish(Event { topic: topic.into(), source: self.id.clone(), payload });
    }

    pub fn subscribe(&self, topic_prefix: &str) -> Subscription {
        self.shared.events.subscribe(topic_prefix)
    }

    /// Registers `<plugin-id>.<name>`; plugins can only publish inside their own namespace.
    pub fn register_command<F, Fut>(&self, name: &str, handler: F) -> Result<()>
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value, BoxError>> + Send + 'static,
    {
        let full = format!("{}.{}", self.id, name);
        if name.is_empty() {
            return Err(KernelError::CommandNamespace { plugin: self.id.clone(), command: full });
        }
        let handler = Arc::new(move |args| -> CommandFuture { Box::pin(handler(args)) });
        self.shared.commands.register(full, handler)
    }

    /// Any plugin may call any command; commands are the public cross-plugin API.
    pub async fn invoke(&self, command: &str, args: Value) -> Result<Value> {
        self.shared.commands.invoke(command, args).await
    }

    pub(crate) fn release(&self) {
        self.shared.commands.remove_prefix(&format!("{}.", self.id));
        self.shared.services.remove_owner(&self.id);
    }
}
