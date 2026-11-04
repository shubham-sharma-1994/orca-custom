use orca_sdk::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Service other plugins resolve via `cx.service::<ConfigStore>()`.
#[derive(Default)]
pub struct ConfigStore {
    values: RwLock<HashMap<String, Value>>,
}

impl ConfigStore {
    pub fn get(&self, key: &str) -> Option<Value> {
        self.values.read().unwrap().get(key).cloned()
    }
    pub fn set(&self, key: &str, value: Value) {
        self.values.write().unwrap().insert(key.into(), value);
    }
}

pub struct CoreConfig;

#[async_trait]
impl Plugin for CoreConfig {
    fn manifest(&self) -> Manifest {
        Manifest::new("core.config", "0.1.0").describe("Key/value configuration store")
    }

    async fn start(&self, cx: &Context) -> Result<(), BoxError> {
        let store = Arc::new(ConfigStore::default());
        cx.provide_service(store.clone());

        let s = store.clone();
        cx.register_command("get", move |args| {
            let s = s.clone();
            async move { Ok(s.get(args["key"].as_str().ok_or("`key` must be a string")?).unwrap_or(Value::Null)) }
        })?;

        let events = cx.clone();
        cx.register_command("set", move |args| {
            let (store, events) = (store.clone(), events.clone());
            async move {
                let key = args["key"].as_str().ok_or("`key` must be a string")?.to_owned();
                store.set(&key, args["value"].clone());
                events.publish("config.changed", json!({ "key": key }));
                Ok(Value::Null)
            }
        })?;
        Ok(())
    }
}
