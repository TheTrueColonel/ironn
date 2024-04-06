use std::env;
use std::time::Instant;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use crate::doc::Doc;
use crate::doc_row::Row;
use crate::ui::ui;

const QUIT_TIMES: u8 = 0;

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub enum CurrentScreen {
    Main,
    Saving,
}

pub struct App {
    pub current_screen: CurrentScreen,
    cursor_position: Position,
    offset: Position,
    terminal_size: Rect,
    document: Doc,
    status_message: StatusMessage,
    should_quit: bool,
    quit_times: u8,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

#[allow(clippy::missing_const_for_fn)]
impl App {
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| ui(f, self))?;

            if self.should_quit {
                return Ok(());
            }

            self.process_keypress()?;
        }
    }
    pub fn process_keypress(&mut self) -> Result<()> {
        if let Event::Key(pressed_key) = event::read()? {
            #[allow(clippy::single_match)]
            match self.current_screen {
                CurrentScreen::Main => match (pressed_key.modifiers, pressed_key.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('x')) => {
                        if self.quit_times > 0 /*&& self.document.is_dirty()*/ {
                            /*self.status_message = crate::editor::StatusMessage::from(format!(
                                "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                                self.quit_times
                            ));*/

                            self.quit_times -= 1;

                            return Ok(())
                        }
                        self.should_quit = true;
                    },
                    (KeyModifiers::CONTROL, KeyCode::Char('o')) => self.write_out(),
                    (_, KeyCode::Enter) => {
                        self.document.insert_newline(&self.cursor_position);
                        self.move_cursor(KeyCode::Right);
                    }
                    (_, KeyCode::Char(c)) => {
                        self.document.insert(&self.cursor_position, c);
                        self.move_cursor(KeyCode::Right);
                    },
                    (_, KeyCode::Delete) => self.document.delete(&self.cursor_position),
                    (_, KeyCode::Backspace) => {
                        if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                            self.move_cursor(KeyCode::Left);
                            self.document.delete(&self.cursor_position);
                        }
                    }
                    (_, KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::PageUp
                    | KeyCode::PageDown
                    | KeyCode::End
                    | KeyCode::Home) => self.move_cursor(pressed_key.code),
                    _ => ()
                },
                CurrentScreen::Saving => match (pressed_key.modifiers, pressed_key.code) {
                    (_, KeyCode::Char('f')) => {
                        self.move_cursor(KeyCode::Right);
                        //self.current_screen = CurrentScreen::Main;
                    },
                    _ => ()
                }
                _ => ()
            }
        }

        self.scroll();

        Ok(())
    }
    pub fn document(&self) -> &Doc {
        &self.document
    }
    pub fn cursor_position(&self) -> &Position {
        &self.cursor_position
    }
    pub fn offset(&self) -> &Position {
        &self.offset
    }
    pub fn status_message(&self) -> &String {
        &self.status_message.text
    }
    pub fn update_bounds(&mut self, rect: Rect) {
        self.terminal_size = rect;
    }
    fn write_out(&mut self) {
        self.current_screen = CurrentScreen::Saving;
        if self.document.file_name.is_none() {
            let new_name = self.prompt("File Name to Write: ", |_,_,_| {}).unwrap_or(None);

            if new_name.is_none() {
                self.status_message = StatusMessage::from("Cancelled".to_owned());
                return;
            }

            self.document.file_name = new_name;
        }

        if self.document.write_out().is_ok() {
            self.status_message = StatusMessage::from("File saves successfully.".to_owned());
        } else {
            self.status_message = StatusMessage::from("Error writing file.".to_owned());
        }
    }
    fn move_cursor(&mut self, key: KeyCode) {
        let terminal_height = self.terminal_size.height as usize;
        let Position { mut x, mut y} = self.cursor_position;
        let height = self.document.len();
        let mut width = self.document.row(y).map_or(0, Row::len);

        match key {
            KeyCode::Up => {
                y = y.saturating_sub(1);
            },
            KeyCode::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            },
            KeyCode::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;

                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            },
            KeyCode::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            },
            KeyCode::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            },
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            KeyCode::Home => x = 0,
            KeyCode::End => x = width,
            _ => ()
        }

        width = self.document.row(y).map_or(0, Row::len);
        
        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }
    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal_size.width as usize;
        let height = self.terminal_size.height as usize;
        let offset = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>> where C: FnMut(&mut Self, KeyEvent, &String) {
        let mut result = String::new();
        
        loop {
            self.status_message = StatusMessage::from(format!("{prompt}{result}"));
            
            if let Event::Key(pressed_key) = event::read()? {
                match (pressed_key.modifiers, pressed_key.code) {
                    (_, KeyCode::Backspace) => result.truncate(result.len().saturating_sub(1)),
                    (_, KeyCode::Enter) => break,
                    (_, KeyCode::Char(c)) => result.push(c),
                    (_, KeyCode::Esc) => {
                        result.truncate(0);
                        break;
                    },
                    _ => ()
                }
                
                callback(self, pressed_key, &result);
            }
        }
        
        self.status_message = StatusMessage::from(String::new());

        self.current_screen = CurrentScreen::Main;
        
        if result.is_empty() {
            return  Ok(None);
        }
        
        Ok(Some(result))
    }
}

impl Default for App {
    fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("Welcome to IronN.");

        let document = args.get(1).map_or_else(Doc::default, |file_name| {
            #[allow(clippy::option_if_let_else)]
            match Doc::open(file_name) { 
                Ok(doc) => {
                    initial_status = format!("Read {} lines.", doc.len());
                    doc
                },
                Err(_) => Doc::default(),
            }
        });

        Self {
            current_screen: CurrentScreen::Main,
            cursor_position: Position::default(),
            offset: Position::default(),
            terminal_size: Rect::default(),
            document,
            status_message: StatusMessage::from(initial_status),
            should_quit: false,
            quit_times: QUIT_TIMES,
        }
    }
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            text: message,
            time: Instant::now(),
        }
    }
}