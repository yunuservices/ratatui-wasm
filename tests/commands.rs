use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_wasm::blit_commands;
use ratatui_wasm::wit::{Alignment, Cell, Color, Line, RenderCommand, Span, Style};

#[test]
fn blit_left_aligned_line() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
    let line = Line {
        y: 1,
        spans: vec![
            Span {
                content: "hi".to_string(),
                style: Some(Style {
                    fg: Some(Color::Green),
                    bg: None,
                    bold: false,
                    italic: false,
                    underline: false,
                }),
            },
            Span {
                content: " there".to_string(),
                style: None,
            },
        ],
        alignment: Some(Alignment::Left),
    };

    blit_commands(
        Rect::new(0, 0, 20, 3),
        &[RenderCommand::Line(line)],
        &mut buf,
    );

    let text: String = buf.content()[20..40]
        .iter()
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert_eq!(text.trim_end(), "hi there");
}

#[test]
fn blit_centered_line() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
    let line = Line {
        y: 0,
        spans: vec![Span {
            content: "center".to_string(),
            style: None,
        }],
        alignment: Some(Alignment::Center),
    };

    blit_commands(
        Rect::new(0, 0, 20, 3),
        &[RenderCommand::Line(line)],
        &mut buf,
    );

    let text: String = buf.content()[0..20]
        .iter()
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert_eq!(text, format!("{}center{}", " ".repeat(7), " ".repeat(7)));
}

#[test]
fn blit_right_aligned_line() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
    let line = Line {
        y: 0,
        spans: vec![Span {
            content: "right".to_string(),
            style: None,
        }],
        alignment: Some(Alignment::Right),
    };

    blit_commands(buf.area, &[RenderCommand::Line(line)], &mut buf);

    let text: String = buf
        .content()
        .iter()
        .map(ratatui_core::buffer::Cell::symbol)
        .collect();
    assert_eq!(text, format!("{}right", " ".repeat(15)));
}

#[test]
fn commands_outside_the_buffer_are_skipped() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 4, 2));
    let area = Rect::new(0, 0, 10, 10);
    let commands = [
        RenderCommand::Cell(Cell {
            x: 8,
            y: 8,
            symbol: "x".to_string(),
            fg: None,
            bg: None,
        }),
        RenderCommand::Line(Line {
            y: 5,
            spans: vec![Span {
                content: "hidden".to_string(),
                style: None,
            }],
            alignment: None,
        }),
    ];

    blit_commands(area, &commands, &mut buf);

    assert_eq!(buf, Buffer::empty(Rect::new(0, 0, 4, 2)));
}
