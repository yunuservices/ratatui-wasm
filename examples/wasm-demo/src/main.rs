use std::path::PathBuf;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::widgets::Block;
use ratatui_wasm::WasmWidget;

const HELLO_RUST_WASM: &str =
    "../wasm-widgets/hello-rust/target/wasm32-wasip2/release/hello_rust.wasm";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let widget_path = std::env::var_os("WASM_WIDGET").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(HELLO_RUST_WASM),
        PathBuf::from,
    );
    let mut counter: i64 = 0;

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| {
                let block = Block::bordered()
                    .title("WASM Widget")
                    .title_bottom(format!(" Counter: {counter} | +/- adjust | q quit "));
                let widget_area = block.inner(frame.area());
                frame.render_widget(block, frame.area());
                frame.render_widget(WasmWidget::from_file(&widget_path, &[]), widget_area);
            })?;

            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    KeyCode::Char('+') | KeyCode::Up => counter += 1,
                    KeyCode::Char('-') | KeyCode::Down => counter -= 1,
                    _ => {}
                }
            }
        }
    })
}
