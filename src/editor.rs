use std::env;
use std::time::{Duration, Instant};
use color_eyre::eyre::Result;
use color_eyre::Report;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::Event::Key;
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crate::{Document, Row, Terminal};

const STATUS_FG_COLOR: Color = Color::Rgb { r: 63, g: 63, b: 63 };
const STATUS_BG_COLOR: Color = Color::Rgb { r: 239, g: 239, b :239 };
const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times: u8,
    highlighted_word: Option<String>,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl Default for Editor {
    fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-Q = quit | Ctrl-S = save | Ctrl-F = find");

        let document = args.get(1).map_or_else(Document::default, |file_name| {
            let doc = Document::open(file_name);

            doc.map_or_else(|_| {
                initial_status = format!("ERR: Could not open file: {file_name}");
                Document::default()
            }, |doc| doc)
        });

        Self {
            should_quit: false,
            terminal: Terminal::instantiate().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position { x: 0, y: 1},
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
            highlighted_word: None,
        }
    }
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(&error);
            }
        }
    }
    fn refresh_screen(&mut self) -> Result<()> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());

        if self.should_quit {
            Terminal::restore();
        } else {
            self.draw_header_bar();
            self.document.highlight(&self.highlighted_word, Some(self.offset.y.saturating_add(self.terminal.size().height as usize)));
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();

            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }

        Terminal::cursor_show();
        Terminal::flush()
    }
    fn save(&mut self) {
        if self.document.file_name.is_none() {
           let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_owned());
                return;
            }
            
            self.document.file_name = new_name;
        }
        
        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_owned());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_owned());
        }
    }
    fn process_keypress(&mut self) -> Result<()> {
        let event = Terminal::read()?;

        if let Key(pressed_key) = event {
            match (pressed_key.modifiers, pressed_key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('q')) | (_, KeyCode::Esc) => {
                    if self.quit_times > 0 && self.document.is_dirty() {
                        self.status_message = StatusMessage::from(format!(
                            "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                            self.quit_times
                        ));

                        self.quit_times -= 1;

                        return Ok(())
                    }
                    self.should_quit = true;
                },
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => self.save(),
                (KeyModifiers::CONTROL, KeyCode::Char('f')) => self.search(),
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
            }
        }

        self.scroll();
        
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }

        Ok(())
    }
    fn move_cursor(&mut self, key: KeyCode) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x} = self.cursor_position;
        let height = self.document.len();
        let mut width = self.document.row(y).map_or(0, Row::len);

        match key {
            KeyCode::Up => {
                if y > 1 {
                    y = y.saturating_sub(1);
                }
            },
            KeyCode::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            },
            KeyCode::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 1 {
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
            },
            KeyCode::Home => x = 0,
            KeyCode::End => x = width,
            _ => (),
        }

        width = self.document.row(y).map_or(0, Row::len);

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }
    fn scroll(&mut self) {
        let Position { x, y} = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
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
    fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);

        println!("{row}\r");
    }
    fn draw_rows(&self) {
        let height = self.terminal.size().height;

        
        
        for terminal_row in 1..height {
            Terminal::clear_current_line();

            // If the row at the current index has text, draw it to screen
            if let Some(row) = self.document.row(self.offset.y.saturating_add(terminal_row as usize)) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
    fn draw_header_bar(&self) {
        Terminal::clear_current_line();
        //let mut info;
        let width = self.terminal.size().width as usize;
        Terminal::set_background_color(Color::White);
        println!("{}\r", " ".repeat(width));

        Terminal::reset_colors();
    }
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hecto editor -- version {VERSION}");
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));

        welcome_message = format!("~{spaces}{welcome_message}");
        welcome_message.truncate(width);

        println!("{welcome_message}\r");
    }
    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };
        let mut file_name = "[No Name]".to_owned();
        
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        
        status = format!("{} - {} lines{}",
                             file_name,
                             self.document.len(),
                             modified_indicator);
        
        let line_indicator = format!(
            "{} | {}/{}",
            self.document.file_type(),
            self.cursor_position.y.saturating_add(1),
            self.document.len(),
        );
        
        let len = status.len() + line_indicator.len();
        
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        
        status = format!("{status}{line_indicator}");
        
        status.truncate(width);
        #[allow(clippy::no_effect)]
        SetBackgroundColor(STATUS_BG_COLOR);
        #[allow(clippy::no_effect)]
        SetForegroundColor(STATUS_FG_COLOR);
        println!("{status}\r");
        Terminal::reset_colors();
    }
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        
        if message.time.elapsed() < Duration::new(5, 0) {
            let mut text = message.text.clone();
            
            text.truncate(self.terminal.size().width as usize);
            
            print!("{text}");
        }
    }
    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>> where C: FnMut(&mut Self, KeyEvent, &String) {
        let mut result = String::new();

        loop {
            self.status_message = StatusMessage::from(format!("{prompt}{result}"));
            self.refresh_screen()?;

            let event = Terminal::read()?;
            
            if let Key(key) = event {
                match key.code {
                    KeyCode::Backspace => result.truncate(result.len().saturating_sub(1)),
                    KeyCode::Enter => break,
                    KeyCode::Char(c) => {
                        if !c.is_control() {
                            result.push(c);
                        }
                    },
                    KeyCode::Esc => {
                        result.truncate(0);
                        break;
                    },
                    _ => (),
                }
                callback(self, key, &result);
            }
        }

        self.status_message = StatusMessage::from(String::new());
        
        if result.is_empty() {
            return Ok(None);
        }
        
        Ok(Some(result))
    }
    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;
        
        let query = self
            .prompt("Search (ESC to cancel, Arrows to navigate): ", |editor, key, query| {
                let mut moved = false;
                
                match key.code { 
                    KeyCode::Right | KeyCode::Down => {
                        direction = SearchDirection::Forward;
                        editor.move_cursor(KeyCode::Right);
                        moved = true;
                    },
                    KeyCode::Left | KeyCode::Up => direction = SearchDirection::Backward,
                    _ => direction = SearchDirection::Forward,
                }
                
                if let Some(position) = editor.document.find(query, &editor.cursor_position, direction) {
                    editor.cursor_position = position;
                    editor.scroll();
                } else if moved {
                    editor.move_cursor(KeyCode::Left);
                }
                
                editor.highlighted_word = Some(query.to_owned());
            }).unwrap_or(None);
        
        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        
        self.highlighted_word = None;
    }
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

/*impl Default for Position{
    fn default() -> Self {
        Self {
            x: 0,
            y: 1,
        }
    }
}*/

fn die(e: &Report) {
    Terminal::clear_screen();
    panic!("{e}");
}