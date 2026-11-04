use crate::manifest::PluginId;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("plugin `{0}` is registered twice")]
    DuplicatePlugin(PluginId),
    #[error("plugin `{plugin}` requires `{dependency}`, which is not registered")]
    MissingDependency { plugin: PluginId, dependency: PluginId },
    #[error("dependency cycle among plugins: {0:?}")]
    DependencyCycle(Vec<PluginId>),
    #[error("plugin `{plugin}` failed to start: {source}")]
    Start { plugin: PluginId, source: BoxError },
    #[error("command `{0}` not found")]
    CommandNotFound(String),
    #[error("command `{command}` is outside plugin namespace `{plugin}`")]
    CommandNamespace { plugin: PluginId, command: String },
    #[error("command `{0}` is already registered")]
    CommandExists(String),
    #[error("command `{command}` failed: {source}")]
    Command { command: String, source: BoxError },
    #[error("plugin `{plugin}` may not access service `{service}`; declare its owner in `requires`")]
    ServiceDenied { plugin: PluginId, service: &'static str },
    #[error("service `{0}` is not provided by any plugin")]
    ServiceMissing(&'static str),
}

pub type Result<T, E = KernelError> = std::result::Result<T, E>;
