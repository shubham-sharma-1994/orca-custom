use crate::error::{KernelError, Result};
use crate::manifest::PluginId;
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct Entry {
    owner: PluginId,
    value: Arc<dyn Any + Send + Sync>,
}

/// Typed in-process services. Visibility is limited to the owner and its dependents.
#[derive(Default)]
pub(crate) struct ServiceRegistry {
    map: RwLock<HashMap<TypeId, Entry>>,
}

impl ServiceRegistry {
    pub(crate) fn provide<T: Any + Send + Sync>(&self, owner: &PluginId, value: Arc<T>) {
        self.map
            .write()
            .unwrap()
            .insert(TypeId::of::<T>(), Entry { owner: owner.clone(), value });
    }
    pub(crate) fn get<T: Any + Send + Sync>(
        &self,
        caller: &PluginId,
        visible_owners: &[PluginId],
    ) -> Result<Arc<T>> {
        let map = self.map.read().unwrap();
        let entry = map.get(&TypeId::of::<T>()).ok_or(KernelError::ServiceMissing(type_name::<T>()))?;
        if &entry.owner != caller && !visible_owners.contains(&entry.owner) {
            return Err(KernelError::ServiceDenied { plugin: caller.clone(), service: type_name::<T>() });
        }
        Ok(entry.value.clone().downcast::<T>().expect("TypeId matched"))
    }
    pub(crate) fn remove_owner(&self, owner: &PluginId) {
        self.map.write().unwrap().retain(|_, e| &e.owner != owner);
    }
}
