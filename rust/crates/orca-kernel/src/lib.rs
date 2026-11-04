//! Orca microkernel: plugin lifecycle, typed services, JSON commands and an event bus.
//! It contains no product features; everything else is a [`Plugin`].

mod commands;
mod error;
mod events;
mod host;
mod manifest;
mod plugin;
mod services;

pub use error::{BoxError, KernelError, Result};
pub use events::{Event, Subscription};
pub use host::{Host, HostBuilder};
pub use manifest::{Manifest, PluginId};
pub use plugin::{Context, Plugin};
