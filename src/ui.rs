use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use crate::app::{App, CurrentScreen};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn ui(f: &mut Frame, app: &mut App) {
    ui_main(f, app);
}

fn ui_main(f: &mut Frame, app: &mut App) {
    let chunks = Layout::new(Direction::Vertical, [
        Constraint::Length(1), // Header
        Constraint::Min(1), // Document
        Constraint::Length(1), // Status Message
        Constraint::Length(2), // Controls
    ]).split(f.size());

    draw_header_bar(f, app, chunks[0]);

    draw_document_rows(f, app, chunks[1]);
    draw_cursor(f, app);

    draw_status(f, app, chunks[2]);
    draw_controls(f, app, chunks[3]);
}

fn draw_header_bar(f: &mut Frame, app: &App, chunk: Rect) {
    let title_text = format!("  IronN {VERSION}");

    let title_chunks = Layout::new(Direction::Horizontal, [
        Constraint::Length(title_text.len() as u16),
        Constraint::Min(1),
    ]).split(chunk);

    let title_block_style = Style::default()
        .fg(Color::Black)
        .bg(Color::White);

    let title = Paragraph::new(Text::styled(
        title_text,
        Style::default()
    )).block(Block::default().style(title_block_style));

    let title_filename = Paragraph::new(Text::styled(
        file_text(app, &title_chunks),
        Style::default()
    )).block(Block::default().style(title_block_style));

    f.render_widget(title, title_chunks[0]);
    f.render_widget(title_filename, title_chunks[1]);
}

fn draw_document_rows(f: &mut Frame, app: &mut App, chunk: Rect) {
    app.update_bounds(chunk);

    let mut rows = Vec::<ListItem>::new();

    for terminal_row in 0..chunk.height as usize {
        if let Some(row) = app.document().row(app.offset().y.saturating_add(terminal_row)) {
            let new_list_item = ListItem::new(Line::from(Span::styled(
                row.as_str(),
                Style::default()
            )));

            rows.push(new_list_item);
        }
    }


    let list = List::new(rows);

    f.render_widget(list, chunk);
}

fn draw_cursor(f: &mut Frame, app: &App) {
    let position = app.cursor_position();
    let offset = app.offset();
    
    let x = position.x.saturating_sub(offset.x) as u16;
    let y = position.y.saturating_sub(offset.y) as u16;

    f.set_cursor(x, y.saturating_add(1));
}

fn draw_status(f: &mut Frame, app: &App, chunk: Rect) {
    match app.current_screen { 
        CurrentScreen::Main => {
            let title_block_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Red);

            let status = Paragraph::new(Text::styled(
            "test",
            Style::default()
            )).block(Block::default().style(title_block_style));

            f.render_widget(status, chunk);
        },
        CurrentScreen::Saving => {
            let title_block_style = Style::default()
                .fg(Color::Black)
                .bg(Color::White);

            let status = Paragraph::new(Text::styled(
                "test",
                Style::default()
            )).block(Block::default().style(title_block_style));

            f.render_widget(status, chunk);
        }
    }
}

fn draw_controls(f: &mut Frame, app: &App, chunk: Rect) {
    let control_chunks = Layout::new(Direction::Vertical, [
        Constraint::Length(1),
        Constraint::Length(1),
    ]).split(chunk);

    let mut test = Line::default();

    test.spans.push(Span::styled(
        "^X",
        Style::from((Color::Black, Color::White))
    ));

    test.spans.push(Span::styled(
        " Exit",
        Style::default()
    ));

    f.render_widget(test, control_chunks[1]);
}

fn file_text(app: &App, areas: &[Rect]) -> String {
    let mut welcome_message: String;

    if let Some(file_name) = &app.document().file_name {
        welcome_message = file_name.to_owned();
    } else {
        welcome_message = "New Buffer".to_owned();
    }

    let width = areas.iter().fold(0, |_, area| area.width) as usize;
    let len = welcome_message.len();
    let padding = width.saturating_sub(len) / 2;
    let spaces = " ".repeat(padding.saturating_sub(4));

    welcome_message = format!("{spaces}{welcome_message}");
    welcome_message.truncate(areas.last().unwrap().width as usize);

    welcome_message
}