//! Plugin-author surface. Plugins depend on this crate only, never on kernel internals.

pub use async_trait::async_trait;
pub use orca_kernel::{BoxError, Context, Event, Manifest, Plugin, PluginId, Subscription};
pub use serde_json::{json, Value};

pub mod prelude {
    pub use super::*;
}
