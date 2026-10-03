use crate::exports::ratatui::widget::widget::{
    Event, Guest, KeyCode, KeyEvent, Line, Rect, RenderCommand, RenderResult, Span, WidgetError,
};

wit_bindgen::generate!({
    path: "../../../wit/widget.wit",
    world: "wasm-widget",
});

const PACKAGE_CAPABILITY: &str = "env:CARGO_PKG_NAME";

struct Probe;

impl Guest for Probe {
    fn render(_area: Rect, state: Option<Vec<u8>>) -> Result<RenderResult, WidgetError> {
        let count = state
            .and_then(|bytes| bytes.try_into().ok())
            .map_or(0, u32::from_le_bytes)
            + 1;
        let package = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
        let path_visible = std::env::var_os("PATH").is_some();

        Ok(RenderResult {
            commands: vec![RenderCommand::Line(Line {
                y: 0,
                spans: vec![Span {
                    content: format!("count={count} package={package} path={path_visible}"),
                    style: None,
                }],
                alignment: None,
            })],
            state: Some(count.to_le_bytes().to_vec()),
        })
    }

    fn handle_event(event: Event) -> Result<bool, WidgetError> {
        Ok(matches!(
            event,
            Event::Key(KeyEvent { code: KeyCode::Codepoint(symbol), .. }) if symbol == "+"
        ))
    }

    fn capabilities() -> Vec<String> {
        vec![PACKAGE_CAPABILITY.to_string()]
    }
}

export!(Probe);
