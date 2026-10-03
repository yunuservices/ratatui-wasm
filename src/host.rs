use std::path::{Path, PathBuf};

use anyhow::Result;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::StatefulWidget;
use wasmtime::Store;
use wasmtime::component::Linker;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::cache;
use crate::generated::WasmWidget as WasmWidgetBinding;
use crate::manifest::PluginManifest;
use crate::rect_to_wit;
use crate::wit::{Event, RenderResult};

pub struct PluginWidget {
    store: Store<WasiState>,
    binding: Box<WasmWidgetBinding>,
    capabilities: Vec<String>,
}

impl PluginWidget {
    /// Loads a `.wasm` component and fails if it requests a capability that was not granted.
    pub fn from_file(path: impl AsRef<Path>, capabilities: &[String]) -> Result<Self> {
        let (engine, component) = cache::load_component(path.as_ref())?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
            .map_err(|e| anyhow::anyhow!("adding WASI to linker: {e}"))?;

        let mut builder = WasiCtxBuilder::new();
        apply_capabilities(&mut builder, capabilities);
        let wasi = builder.build();
        let mut store = Store::new(&engine, WasiState::new(wasi));

        let binding = Box::new(
            WasmWidgetBinding::instantiate(&mut store, &component, &linker)
                .map_err(|e| anyhow::anyhow!("instantiating wasm widget component: {e}"))?,
        );

        let requested = binding
            .ratatui_widget_widget()
            .call_capabilities(&mut store)
            .map_err(|e| anyhow::anyhow!("calling widget capabilities: {e}"))?;

        if let Some(denied) = requested.iter().find(|c| !capabilities.contains(c)) {
            anyhow::bail!("widget requires capability `{denied}` which was not granted");
        }

        Ok(Self {
            store,
            binding,
            capabilities: capabilities.to_vec(),
        })
    }

    /// Loads the widget described by a `ratatui.plugin.toml` manifest, granting only the requested
    /// capabilities that appear in `allowed`.
    pub fn from_manifest(path: impl AsRef<Path>, allowed: &[String]) -> Result<Self> {
        let manifest = PluginManifest::from_file(&path)?;
        let manifest_dir = path.as_ref().parent().unwrap_or_else(|| Path::new("."));
        let entry = manifest.resolve_entry(manifest_dir);
        Self::from_file(entry, &manifest.grant(allowed)?)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let mut state = Vec::new();
        self.render_stateful(area, buf, &mut state)
    }

    /// Renders the widget and stores the opaque state it returns in `state`.
    pub fn render_stateful(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Vec<u8>,
    ) -> Result<()> {
        let previous_state = (!state.is_empty()).then_some(state.as_slice());
        let RenderResult {
            commands,
            state: next_state,
        } = self
            .binding
            .ratatui_widget_widget()
            .call_render(&mut self.store, rect_to_wit(area), previous_state)
            .map_err(|e| anyhow::anyhow!("calling widget render: {e}"))?
            .map_err(|e| anyhow::anyhow!("widget render failed: {e:?}"))?;
        if let Some(next_state) = next_state {
            *state = next_state;
        }
        crate::blit_commands(area, &commands, buf);
        Ok(())
    }

    /// Delivers an input event and returns `true` if the widget consumed it.
    pub fn handle_event(&mut self, event: &Event) -> Result<bool> {
        self.binding
            .ratatui_widget_widget()
            .call_handle_event(&mut self.store, event)
            .map_err(|e| anyhow::anyhow!("calling widget handle_event: {e}"))?
            .map_err(|e| anyhow::anyhow!("widget handle_event failed: {e:?}"))
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

/// A [`ratatui_core::widgets::Widget`] backed by a WASM plugin.
///
/// The component is instantiated on every render, so a rebuilt `.wasm` file is picked up on the
/// next frame.
#[derive(Clone, Debug)]
pub struct WasmWidget {
    path: PathBuf,
    capabilities: Vec<String>,
}

impl WasmWidget {
    pub fn from_file(path: impl AsRef<Path>, capabilities: &[String]) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            capabilities: capabilities.to_vec(),
        }
    }

    pub fn from_manifest(path: impl AsRef<Path>, allowed: &[String]) -> Result<Self> {
        let manifest = PluginManifest::from_file(&path)?;
        let manifest_dir = path.as_ref().parent().unwrap_or_else(|| Path::new("."));
        Ok(Self::from_file(
            manifest.resolve_entry(manifest_dir),
            &manifest.grant(allowed)?,
        ))
    }
}

impl ratatui_core::widgets::Widget for WasmWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match PluginWidget::from_file(&self.path, &self.capabilities) {
            Ok(mut plugin) => {
                if let Err(err) = plugin.render(area, buf) {
                    tracing::debug!("failed to render WASM widget: {err:#}");
                }
            }
            Err(err) => {
                tracing::debug!("failed to load WASM widget: {err:#}");
            }
        }
    }
}

/// A [`StatefulWidget`] backed by a WASM plugin that keeps opaque state between frames.
#[derive(Clone, Debug)]
pub struct StatefulWasmWidget {
    path: PathBuf,
    capabilities: Vec<String>,
}

impl StatefulWasmWidget {
    pub fn from_file(path: impl AsRef<Path>, capabilities: &[String]) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            capabilities: capabilities.to_vec(),
        }
    }

    pub fn from_manifest(path: impl AsRef<Path>, allowed: &[String]) -> Result<Self> {
        let manifest = PluginManifest::from_file(&path)?;
        let manifest_dir = path.as_ref().parent().unwrap_or_else(|| Path::new("."));
        Ok(Self::from_file(
            manifest.resolve_entry(manifest_dir),
            &manifest.grant(allowed)?,
        ))
    }
}

impl StatefulWidget for StatefulWasmWidget {
    type State = Vec<u8>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Vec<u8>) {
        match PluginWidget::from_file(&self.path, &self.capabilities) {
            Ok(mut plugin) => {
                if let Err(err) = plugin.render_stateful(area, buf, state) {
                    tracing::debug!("failed to render stateful WASM widget: {err:#}");
                }
            }
            Err(err) => {
                tracing::debug!("failed to load stateful WASM widget: {err:#}");
            }
        }
    }
}

fn apply_capabilities(builder: &mut WasiCtxBuilder, capabilities: &[String]) {
    if capabilities.iter().any(|c| c == "stdio:stdout") {
        builder.inherit_stdout();
    }
    if capabilities.iter().any(|c| c == "stdio:stderr") {
        builder.inherit_stderr();
    }
    if capabilities.iter().any(|c| c == "stdio:stdin") {
        builder.inherit_stdin();
    }
    if capabilities.iter().any(|c| c == "env:read") {
        builder.inherit_env();
    }
}

struct WasiState {
    ctx: WasiCtx,
    table: wasmtime::component::ResourceTable,
}

impl WasiState {
    fn new(ctx: WasiCtx) -> Self {
        Self {
            ctx,
            table: wasmtime::component::ResourceTable::new(),
        }
    }
}

impl WasiView for WasiState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}
