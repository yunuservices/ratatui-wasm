use crate::exports::ratatui::widget::widget::{
    Cell, Color, Event, Guest, Rect, RenderCommand, RenderResult, RgbColor, WidgetError,
};

wit_bindgen::generate!({
    path: "../../../wit/widget.wit",
    world: "wasm-widget",
});

const GREETING: &str = "Hello from WASM";
const FOREGROUND: RgbColor = RgbColor { r: 0, g: 255, b: 0 };
const BACKGROUND: RgbColor = RgbColor { r: 0, g: 0, b: 0 };

struct HelloWidget;

impl Guest for HelloWidget {
    fn render(area: Rect, _state: Option<Vec<u8>>) -> Result<RenderResult, WidgetError> {
        let commands = GREETING
            .chars()
            .zip(0..area.width)
            .map(|(symbol, x)| {
                RenderCommand::Cell(Cell {
                    x,
                    y: 0,
                    symbol: symbol.to_string(),
                    fg: Some(Color::Rgb(FOREGROUND)),
                    bg: Some(Color::Rgb(BACKGROUND)),
                })
            })
            .collect();

        Ok(RenderResult {
            commands,
            state: None,
        })
    }

    fn handle_event(_event: Event) -> Result<bool, WidgetError> {
        Ok(false)
    }

    fn capabilities() -> Vec<String> {
        Vec::new()
    }
}

export!(HelloWidget);
