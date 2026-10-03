use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, SystemTime};

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_wasm::{Limits, PluginWidget, StatefulWasmWidget, WasmWidget, event};

const HELLO_RUST_MANIFEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/wasm-widgets/hello-rust/ratatui.plugin.toml"
);
const HELLO_RUST_WASM: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/wasm-widgets/hello-rust/target/wasm32-wasip2/release/hello_rust.wasm"
);
const HELLO_C_WASM: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/wasm-widgets/hello-c/hello_c.wasm"
);
const PROBE_WASM: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/probe/target/wasm32-wasip2/release/probe.wasm"
);
const PROBE_CAPABILITY: &str = "env:CARGO_PKG_NAME";

fn probe_capabilities() -> Vec<String> {
    vec![PROBE_CAPABILITY.to_string()]
}

fn first_line(buf: &Buffer) -> String {
    let width = usize::from(buf.area.width);
    let line: String = buf.content()[..width]
        .iter()
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    line.trim_end().to_string()
}

#[test]
fn loads_and_renders_hello_rust() {
    let mut widget = PluginWidget::from_file(HELLO_RUST_WASM, &[]).expect("widget loads");
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
    widget
        .render(Rect::new(0, 0, 40, 3), &mut buf)
        .expect("widget renders");

    let line: String = buf
        .content()
        .iter()
        .take(40)
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert!(
        line.contains("Hello from WASM"),
        "expected rendered text, got: {line:?}"
    );
}

#[test]
fn wasm_widget_wrapper_renders_via_widget_trait() {
    let widget = WasmWidget::from_file(HELLO_RUST_WASM, &[]);
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
    widget.render(Rect::new(0, 0, 40, 3), &mut buf);

    let line: String = buf
        .content()
        .iter()
        .take(40)
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert!(
        line.contains("Hello from WASM"),
        "expected rendered text via WasmWidget wrapper, got: {line:?}"
    );
}

#[test]
fn hello_rust_ignores_key_events() {
    let mut widget = PluginWidget::from_file(HELLO_RUST_WASM, &[]).expect("widget loads");

    let event = event::key(event::char_key('q'), 0);
    let handled = widget
        .handle_event(&event)
        .expect("widget can handle events");
    assert!(!handled, "hello-rust does not claim key events");
}

#[test]
fn stateless_guest_leaves_state_empty() {
    let widget = StatefulWasmWidget::from_file(HELLO_RUST_WASM, &[]);
    let mut state: Vec<u8> = Vec::new();
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
    StatefulWidget::render(widget, Rect::new(0, 0, 40, 3), &mut buf, &mut state);

    let line: String = buf
        .content()
        .iter()
        .take(40)
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert!(
        line.contains("Hello from WASM"),
        "expected rendered text via StatefulWasmWidget, got: {line:?}"
    );
    assert_eq!(state, Vec::<u8>::new(), "hello-rust does not emit state");
}

#[test]
fn loads_widget_from_manifest() {
    let mut widget =
        PluginWidget::from_manifest(HELLO_RUST_MANIFEST, &[]).expect("manifest loads widget");
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
    widget
        .render(Rect::new(0, 0, 40, 3), &mut buf)
        .expect("render");

    let line: String = buf
        .content()
        .iter()
        .take(40)
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert!(
        line.contains("Hello from WASM"),
        "expected rendered text from manifest, got: {line:?}"
    );
}

#[test]
fn wasm_widget_from_manifest_renders() {
    let widget =
        WasmWidget::from_manifest(HELLO_RUST_MANIFEST, &[]).expect("manifest creates widget");
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
    ratatui_core::widgets::Widget::render(widget, Rect::new(0, 0, 40, 3), &mut buf);

    let line: String = buf
        .content()
        .iter()
        .take(40)
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert!(
        line.contains("Hello from WASM"),
        "expected rendered text from WasmWidget manifest, got: {line:?}"
    );
}

#[test]
fn c_guest_renders() {
    if !Path::new(HELLO_C_WASM).exists() {
        return;
    }
    let mut widget = PluginWidget::from_file(HELLO_C_WASM, &[]).expect("C widget loads");
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
    widget
        .render(Rect::new(0, 0, 40, 3), &mut buf)
        .expect("C widget renders");

    let line: String = buf
        .content()
        .iter()
        .take(40)
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert!(
        line.contains("Hello from C"),
        "expected rendered text from C guest, got: {line:?}"
    );
}

#[test]
fn from_file_fails_when_wasm_missing() {
    let result = PluginWidget::from_file("/tmp/ratatui-wasm-missing.wasm", &[]);
    assert!(result.is_err());
}

#[test]
fn from_manifest_fails_when_manifest_missing() {
    let result = PluginWidget::from_manifest("/tmp/ratatui-wasm-missing.toml", &[]);
    assert!(result.is_err());
}

#[test]
fn wasm_widget_from_manifest_fails_when_missing() {
    let result = WasmWidget::from_manifest("/tmp/ratatui-wasm-missing.toml", &[]);
    assert!(result.is_err());
}

#[test]
fn widget_fails_with_invalid_wasm_file() {
    let invalid_wasm =
        std::env::temp_dir().join(format!("ratatui-wasm-bad-{}", std::process::id()));
    fs::write(&invalid_wasm, b"not a wasm component").unwrap();
    let result = PluginWidget::from_file(&invalid_wasm, &[]);
    assert!(result.is_err());
    let _ = fs::remove_file(&invalid_wasm);
}

#[test]
fn widget_fails_when_fuel_runs_out() {
    let limits = Limits {
        fuel_per_call: 1,
        ..Limits::default()
    };
    assert!(PluginWidget::from_file_with_limits(HELLO_RUST_WASM, &[], limits).is_err());
}

#[test]
fn widget_fails_when_memory_limit_is_too_small() {
    let limits = Limits {
        memory_bytes: 64 * 1024,
        ..Limits::default()
    };
    assert!(PluginWidget::from_file_with_limits(HELLO_RUST_WASM, &[], limits).is_err());
}

#[test]
fn probe_is_refused_without_its_capability() {
    let Err(err) = PluginWidget::from_file(PROBE_WASM, &[]) else {
        panic!("probe loaded without its capability");
    };
    assert!(format!("{err:#}").contains(PROBE_CAPABILITY), "got {err:#}");
}

#[test]
fn probe_sees_only_granted_environment_variables() {
    let mut widget =
        PluginWidget::from_file(PROBE_WASM, &probe_capabilities()).expect("probe loads");
    let mut buf = Buffer::empty(Rect::new(0, 0, 60, 1));
    widget.render(buf.area, &mut buf).expect("probe renders");
    assert_eq!(first_line(&buf), "count=1 package=ratatui-wasm path=false");
}

#[test]
fn stateful_widget_persists_state_between_frames() {
    let mut state = Vec::new();
    let mut buf = Buffer::empty(Rect::new(0, 0, 60, 1));
    for _ in 0..2 {
        StatefulWasmWidget::from_file(PROBE_WASM, &probe_capabilities())
            .render(buf.area, &mut buf, &mut state);
    }
    assert_eq!(first_line(&buf), "count=2 package=ratatui-wasm path=false");
    assert_eq!(state, 2u32.to_le_bytes());
}

#[test]
fn probe_consumes_only_the_plus_key() {
    let mut widget =
        PluginWidget::from_file(PROBE_WASM, &probe_capabilities()).expect("probe loads");
    let plus = event::key(event::char_key('+'), 0);
    let quit = event::key(event::char_key('q'), 0);
    assert!(widget.handle_event(&plus).expect("probe handles +"));
    assert!(!widget.handle_event(&quit).expect("probe handles q"));
}

#[test]
fn widget_reloads_when_the_file_changes() {
    let plugin_path =
        std::env::temp_dir().join(format!("ratatui-wasm-reload-{}.wasm", std::process::id()));
    let mut buf = Buffer::empty(Rect::new(0, 0, 60, 1));

    replace_plugin(&plugin_path, HELLO_RUST_WASM, 1);
    WasmWidget::from_file(&plugin_path, &probe_capabilities()).render(buf.area, &mut buf);
    assert_eq!(first_line(&buf), "Hello from WASM");

    replace_plugin(&plugin_path, PROBE_WASM, 2);
    buf.reset();
    WasmWidget::from_file(&plugin_path, &probe_capabilities()).render(buf.area, &mut buf);
    assert_eq!(first_line(&buf), "count=1 package=ratatui-wasm path=false");

    let _ = fs::remove_file(&plugin_path);
}

/// Copies `source` over `target` with an explicit modification time, so the reload does not
/// depend on the file system's timestamp resolution.
fn replace_plugin(target: &Path, source: &str, modified_secs: u64) {
    fs::copy(source, target).expect("plugin copied");
    File::options()
        .write(true)
        .open(target)
        .and_then(|file| {
            file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(modified_secs))
        })
        .expect("modification time set");
}
