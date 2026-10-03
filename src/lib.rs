use ratatui_core::buffer::{Buffer, Cell};
use ratatui_core::layout::{Alignment, Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::text::{Line, Span};
use ratatui_core::widgets::Widget;

use crate::generated::exports::ratatui::widget::widget::{
    Alignment as WitAlignment, Cell as WitCell, Color as WitColor, Line as WitLine,
    Rect as WitRect, RenderCommand, Span as WitSpan, Style as WitStyle,
};

pub mod commands;
pub mod event;
pub mod host;
pub mod manifest;

mod cache;

pub use host::{Limits, PluginWidget, StatefulWasmWidget, WasmWidget};

pub mod wit {
    pub use super::generated::exports::ratatui::widget::widget::{
        Alignment, Cell, Color, Event, KeyCode, KeyEvent, Line, Rect, RenderCommand, RenderResult,
        ResizeEvent, RgbColor, Span, Style, WidgetError,
    };
}

mod generated {
    wasmtime::component::bindgen!({
        path: "wit/widget.wit",
        world: "wasm-widget",
    });
}

pub const fn rect_to_wit(area: Rect) -> WitRect {
    WitRect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height,
    }
}

pub const fn convert_color(color: WitColor) -> Color {
    match color {
        WitColor::Reset => Color::Reset,
        WitColor::Black => Color::Black,
        WitColor::Red => Color::Red,
        WitColor::Green => Color::Green,
        WitColor::Yellow => Color::Yellow,
        WitColor::Blue => Color::Blue,
        WitColor::Magenta => Color::Magenta,
        WitColor::Cyan => Color::Cyan,
        WitColor::Gray => Color::Gray,
        WitColor::DarkGray => Color::DarkGray,
        WitColor::LightRed => Color::LightRed,
        WitColor::LightGreen => Color::LightGreen,
        WitColor::LightYellow => Color::LightYellow,
        WitColor::LightBlue => Color::LightBlue,
        WitColor::LightMagenta => Color::LightMagenta,
        WitColor::LightCyan => Color::LightCyan,
        WitColor::White => Color::White,
        WitColor::Rgb(rgb) => Color::Rgb(rgb.r, rgb.g, rgb.b),
    }
}

pub const fn convert_style(wit_style: &WitStyle) -> Style {
    let mut style = Style::new();
    if let Some(fg) = &wit_style.fg {
        style = style.fg(convert_color(*fg));
    }
    if let Some(bg) = &wit_style.bg {
        style = style.bg(convert_color(*bg));
    }
    if wit_style.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if wit_style.italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if wit_style.underline {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

fn convert_span(wit_span: &WitSpan) -> Span<'static> {
    match &wit_span.style {
        Some(style) => Span::styled(wit_span.content.clone(), convert_style(style)),
        None => Span::raw(wit_span.content.clone()),
    }
}

const fn convert_alignment(alignment: WitAlignment) -> Alignment {
    match alignment {
        WitAlignment::Left => Alignment::Left,
        WitAlignment::Center => Alignment::Center,
        WitAlignment::Right => Alignment::Right,
    }
}

/// Draws the commands returned by a guest into `buf`, clipped to `area`.
pub fn blit_commands(area: Rect, commands: &[RenderCommand], buf: &mut Buffer) {
    for command in commands {
        match command {
            RenderCommand::Cell(wit_cell) => blit_cell(area, wit_cell, buf),
            RenderCommand::Line(wit_line) => blit_line(area, wit_line, buf),
        }
    }
}

fn blit_cell(area: Rect, wit_cell: &WitCell, buf: &mut Buffer) {
    let x = area.x.saturating_add(wit_cell.x);
    let y = area.y.saturating_add(wit_cell.y);
    if x >= area.right() || y >= area.bottom() || !buf.area.contains(Position { x, y }) {
        return;
    }

    let mut style = Style::new();
    if let Some(fg) = &wit_cell.fg {
        style = style.fg(convert_color(*fg));
    }
    if let Some(bg) = &wit_cell.bg {
        style = style.bg(convert_color(*bg));
    }

    let mut cell = Cell::default();
    cell.set_symbol(&wit_cell.symbol);
    cell.set_style(style);
    buf[(x, y)] = cell;
}

fn blit_line(area: Rect, wit_line: &WitLine, buf: &mut Buffer) {
    let y = area.y.saturating_add(wit_line.y);
    if y >= area.bottom() {
        return;
    }

    let spans: Vec<Span<'static>> = wit_line.spans.iter().map(convert_span).collect();
    let mut line = Line::from(spans);
    if let Some(alignment) = wit_line.alignment {
        line = line.alignment(convert_alignment(alignment));
    }

    line.render(Rect::new(area.x, y, area.width, 1), buf);
}
