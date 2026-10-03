use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};
use std::time::SystemTime;

use anyhow::{Context, Result};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine};

use crate::generated::WasmWidgetPre;
use crate::host::WasiState;

struct CachedWidget {
    widget_pre: WasmWidgetPre<WasiState>,
    mtime: SystemTime,
}

static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
    Engine::new(Config::new().consume_fuel(true)).expect("wasmtime supports fuel on this platform")
});

static WIDGET_CACHE: LazyLock<Mutex<HashMap<PathBuf, CachedWidget>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Returns the pre-linked widget at `path`, compiling and linking it again only if the file
/// changed.
pub(crate) fn load_widget(path: &Path) -> Result<WasmWidgetPre<WasiState>> {
    let mtime = fs::metadata(path)
        .and_then(|m| m.modified())
        .with_context(|| format!("reading metadata for {}", path.display()))?;

    if let Some(cached) = lock_cache().get(path)
        && cached.mtime == mtime
    {
        return Ok(cached.widget_pre.clone());
    }

    let component = Component::from_file(&ENGINE, path)
        .map_err(|e| anyhow::anyhow!("loading wasm component from {}: {e}", path.display()))?;
    let mut linker = Linker::new(&ENGINE);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
        .map_err(|e| anyhow::anyhow!("adding WASI to linker: {e}"))?;
    let widget_pre = linker
        .instantiate_pre(&component)
        .and_then(WasmWidgetPre::new)
        .map_err(|e| anyhow::anyhow!("linking wasm widget component: {e}"))?;

    lock_cache().insert(
        path.to_path_buf(),
        CachedWidget {
            widget_pre: widget_pre.clone(),
            mtime,
        },
    );

    Ok(widget_pre)
}

/// The cache only holds finished entries, so a panic while it was locked cannot leave it
/// half-written.
fn lock_cache() -> MutexGuard<'static, HashMap<PathBuf, CachedWidget>> {
    WIDGET_CACHE.lock().unwrap_or_else(PoisonError::into_inner)
}
