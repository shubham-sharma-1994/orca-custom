use crate::error::{BoxError, KernelError, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

pub type CommandFuture = Pin<Box<dyn Future<Output = Result<Value, BoxError>> + Send>>;
pub(crate) type Handler = Arc<dyn Fn(Value) -> CommandFuture + Send + Sync>;

/// JSON-in/JSON-out so one command surface serves CLI, RPC and out-of-process plugins alike.
#[derive(Default)]
pub(crate) struct CommandRegistry {
    handlers: RwLock<BTreeMap<String, Handler>>,
}

impl CommandRegistry {
    pub(crate) fn register(&self, name: String, handler: Handler) -> Result<()> {
        let mut map = self.handlers.write().unwrap();
        if map.contains_key(&name) {
            return Err(KernelError::CommandExists(name));
        }
        map.insert(name, handler);
        Ok(())
    }
    pub(crate) fn remove_prefix(&self, prefix: &str) {
        self.handlers.write().unwrap().retain(|k, _| !k.starts_with(prefix));
    }
    pub(crate) fn names(&self) -> Vec<String> {
        self.handlers.read().unwrap().keys().cloned().collect()
    }
    pub(crate) async fn invoke(&self, name: &str, args: Value) -> Result<Value> {
        let handler = self
            .handlers
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| KernelError::CommandNotFound(name.into()))?;
        handler(args)
            .await
            .map_err(|source| KernelError::Command { command: name.into(), source })
    }
}
