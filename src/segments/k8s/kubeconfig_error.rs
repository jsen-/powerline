use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
// Redundant with the error messages and machine names
#[allow(missing_docs)]
/// Possible errors when loading config
pub enum ConfigError {
    #[error("Failed to determine current context")]
    CurrentContextNotSet,

    #[error("Merging kubeconfig with mismatching kind")]
    KindMismatch,
    #[error("Merging kubeconfig with mismatching apiVersion")]
    ApiVersionMismatch,

    #[error("Unable to load current context: {context_name}")]
    LoadContext { context_name: String },
    #[error("Unable to load cluster of context: {cluster_name}")]
    LoadClusterOfContext { cluster_name: String },

    #[error("Failed to read '{path:?}': {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse Kubeconfig YAML: {0}")]
    ParseYaml(#[source] serde_yaml::Error),

    #[error("Failed to find a single YAML document in Kubeconfig: {0}")]
    EmptyKubeconfig(PathBuf),
}
