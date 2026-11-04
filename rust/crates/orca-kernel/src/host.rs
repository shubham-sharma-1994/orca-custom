use crate::error::{KernelError, Result};
use crate::manifest::{Manifest, PluginId};
use crate::plugin::{Context, Plugin, Shared};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

struct Slot {
    plugin: Box<dyn Plugin>,
    manifest: Manifest,
    cx: Context,
}

#[derive(Default)]
pub struct HostBuilder {
    plugins: Vec<Box<dyn Plugin>>,
}

impl HostBuilder {
    pub fn plugin(mut self, plugin: impl Plugin + 'static) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// Validates the graph and fixes the start order; runs no plugin code.
    pub fn build(self) -> Result<Host> {
        let shared = Arc::new(Shared {
            services: Default::default(),
            commands: Default::default(),
            events: crate::events::EventBus::new(),
        });
        let mut by_id: BTreeMap<PluginId, Slot> = BTreeMap::new();
        for plugin in self.plugins {
            let manifest = plugin.manifest();
            let cx = Context::new(&manifest, shared.clone());
            let id = manifest.id.clone();
            if by_id.insert(id.clone(), Slot { plugin, manifest, cx }).is_some() {
                return Err(KernelError::DuplicatePlugin(id));
            }
        }
        for slot in by_id.values() {
            for dep in &slot.manifest.requires {
                if !by_id.contains_key(dep) {
                    return Err(KernelError::MissingDependency {
                        plugin: slot.manifest.id.clone(),
                        dependency: dep.clone(),
                    });
                }
            }
        }
        let order = topo_order(&by_id)?;
        Ok(Host { shared, slots: by_id, order, started: Vec::new() })
    }
}

fn topo_order(slots: &BTreeMap<PluginId, Slot>) -> Result<Vec<PluginId>> {
    let mut done: BTreeSet<PluginId> = BTreeSet::new();
    let mut order = Vec::with_capacity(slots.len());
    while order.len() < slots.len() {
        let ready: Vec<_> = slots
            .iter()
            .filter(|(id, s)| !done.contains(*id) && s.manifest.requires.iter().all(|d| done.contains(d)))
            .map(|(id, _)| id.clone())
            .collect();
        if ready.is_empty() {
            let stuck = slots.keys().filter(|id| !done.contains(*id)).cloned().collect();
            return Err(KernelError::DependencyCycle(stuck));
        }
        for id in ready {
            done.insert(id.clone());
            order.push(id);
        }
    }
    Ok(order)
}

pub struct Host {
    shared: Arc<Shared>,
    slots: BTreeMap<PluginId, Slot>,
    order: Vec<PluginId>,
    started: Vec<PluginId>,
}

impl Host {
    pub fn builder() -> HostBuilder {
        HostBuilder::default()
    }

    /// Starts plugins dependencies-first. On failure, already-started plugins are stopped.
    pub async fn start(&mut self) -> Result<()> {
        for id in self.order.clone() {
            if self.started.contains(&id) {
                continue;
            }
            let slot = &self.slots[&id];
            if let Err(source) = slot.plugin.start(&slot.cx).await {
                slot.cx.release();
                self.shutdown().await;
                return Err(KernelError::Start { plugin: id, source });
            }
            self.started.push(id);
        }
        Ok(())
    }

    /// Stops plugins dependents-first. Stop errors are ignored so every plugin gets its turn.
    pub async fn shutdown(&mut self) {
        while let Some(id) = self.started.pop() {
            let slot = &self.slots[&id];
            let _ = slot.plugin.stop(&slot.cx).await;
            slot.cx.release();
        }
    }

    pub async fn invoke(&self, command: &str, args: Value) -> Result<Value> {
        self.shared.commands.invoke(command, args).await
    }

    pub fn commands(&self) -> Vec<String> {
        self.shared.commands.names()
    }

    /// Manifests in start order.
    pub fn plugins(&self) -> Vec<&Manifest> {
        self.order.iter().map(|id| &self.slots[id].manifest).collect()
    }
}
