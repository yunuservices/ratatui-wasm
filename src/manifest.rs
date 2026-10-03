use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
    #[serde(default)]
    pub capabilities: Capabilities,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    /// Path to the `.wasm` component, relative to the manifest directory.
    pub entry: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Capabilities {
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
}

impl PluginManifest {
    pub fn parse(content: &str) -> Result<Self> {
        toml::from_str(content).context("parsing plugin manifest")
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("reading {}", path.as_ref().display()))?;
        Self::parse(&content)
    }

    /// Resolves `entry` against `manifest_dir` unless it is already absolute.
    pub fn resolve_entry(&self, manifest_dir: impl AsRef<Path>) -> PathBuf {
        let entry = &self.plugin.entry;
        if entry.is_absolute() {
            entry.clone()
        } else {
            manifest_dir.as_ref().join(entry)
        }
    }

    /// Capabilities to grant the plugin. Optional capabilities are not granted yet.
    pub fn granted_capabilities(&self) -> Vec<String> {
        self.capabilities.required.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest() {
        let content = r#"
[plugin]
name = "hello-rust"
version = "0.1.0"
author = "yunuservices"
entry = "target/wasm32-wasip2/release/hello_rust.wasm"

[capabilities]
required = ["stdio:stdout"]
optional = ["clock:read"]
"#;
        let manifest = PluginManifest::parse(content).unwrap();
        assert_eq!(manifest.plugin.name, "hello-rust");
        assert_eq!(manifest.plugin.version, "0.1.0");
        assert_eq!(manifest.plugin.author.as_deref(), Some("yunuservices"));
        assert_eq!(
            manifest.plugin.entry,
            PathBuf::from("target/wasm32-wasip2/release/hello_rust.wasm")
        );
        assert_eq!(manifest.capabilities.required, vec!["stdio:stdout"]);
        assert_eq!(manifest.capabilities.optional, vec!["clock:read"]);
        assert_eq!(
            manifest.resolve_entry("/tmp"),
            PathBuf::from("/tmp/target/wasm32-wasip2/release/hello_rust.wasm")
        );
    }

    #[test]
    fn manifest_without_optional_caps() {
        let content = r#"
[plugin]
name = "minimal"
version = "1.0.0"
entry = "plugin.wasm"
"#;
        let manifest = PluginManifest::parse(content).unwrap();
        assert_eq!(manifest.capabilities.required, Vec::<String>::new());
        assert_eq!(manifest.capabilities.optional, Vec::<String>::new());
        assert_eq!(manifest.granted_capabilities(), Vec::<String>::new());
    }
}
