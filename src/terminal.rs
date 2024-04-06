use std::io::{stdout, Write};
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use crossterm::{cursor, ExecutableCommand, terminal};
use crossterm::event::{Event, read};
use crossterm::style::{Color, Colors, ResetColor, SetBackgroundColor, SetColors, SetForegroundColor};
use crate::Position;

pub struct Size {
    pub width: u16,
    pub height: u16,
}
pub struct Terminal {
    size: Size
}

impl Terminal {
    /// # Errors
    ///
    /// Will return `Err` if the terminal's size cannot be read, or we fail to enable raw mode
    pub fn instantiate() -> Result<Self> {
        let size = terminal::size()?;
        terminal::enable_raw_mode().ok();

        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
        })
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn size(&self) -> &Size {
        &self.size
    }
    pub fn restore() {
        Self::reset_colors();
        Self::clear_screen();
        terminal::disable_raw_mode().ok();
    }
    pub fn clear_screen() {
        stdout().execute(terminal::Clear(terminal::ClearType::All)).ok();
    }
    pub fn clear_current_line() {
        stdout().execute(terminal::Clear(terminal::ClearType::CurrentLine)).ok();
    }
    /// # Panics
    /// 
    /// Will panic if either value in `&Position` gets truncated when converting to a `u16`
    pub fn cursor_position(position: &Position) {
        let Position { mut x, mut y} = position;
        
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        
        let x = u16::try_from(x).unwrap();
        let y = u16::try_from(y).unwrap();

        print!("{}", cursor::MoveTo(x - 1, y - 1));
    }
    /// # Errors
    ///
    /// Will return `Err` if the method failed to flush stdout
    pub fn flush() -> Result<()> {
        stdout().flush().wrap_err("Failed to flush stdout")
    }
    /// # Errors
    ///
    /// Will return `Err` if the method failed to read the input key
    pub fn read() -> Result<Event> {
        read().wrap_err("Failed to read key")
    }
    pub fn cursor_hide() {
        stdout().execute(cursor::DisableBlinking).ok();
    }
    pub fn cursor_show() {
        stdout().execute(cursor::EnableBlinking).ok();
    }
    pub fn set_foreground_color(color: Color) {
        stdout().execute(SetForegroundColor(color)).ok();
    }
    pub fn set_background_color(color: Color) {
        stdout().execute(SetBackgroundColor(color)).ok();
    }
    pub fn set_colors(colors: Colors) {
        stdout().execute(SetColors(colors)).ok();
    }
    pub fn reset_colors() {
        stdout().execute(ResetColor).ok();
    }
}