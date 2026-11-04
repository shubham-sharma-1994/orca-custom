use std::fmt;
use std::sync::Arc;

/// Stable dotted identifier, e.g. `core.config`. Also the command namespace.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PluginId(Arc<str>);

impl PluginId {
    pub fn new(id: impl AsRef<str>) -> Self {
        Self(Arc::from(id.as_ref()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl fmt::Debug for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Declarative description of a plugin; the kernel reads it before running any plugin code.
#[derive(Clone, Debug)]
pub struct Manifest {
    pub id: PluginId,
    pub version: String,
    pub description: String,
    /// Plugins that must start first. Only their services are visible to this plugin.
    pub requires: Vec<PluginId>,
}

impl Manifest {
    pub fn new(id: &str, version: &str) -> Self {
        Self {
            id: PluginId::new(id),
            version: version.into(),
            description: String::new(),
            requires: Vec::new(),
        }
    }
    pub fn describe(mut self, text: &str) -> Self {
        self.description = text.into();
        self
    }
    pub fn requires(mut self, id: &str) -> Self {
        self.requires.push(PluginId::new(id));
        self
    }
}
