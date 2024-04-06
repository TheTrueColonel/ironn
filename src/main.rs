#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![allow(
    non_snake_case,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else,
    clippy::single_call_fn,
    clippy::print_stdout,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums
)]
mod editor;
mod terminal;
mod row;
mod document;
mod highlighting;
mod filetype;
mod app;
mod ui;
mod doc;
mod doc_row;

use std::error::Error;
use std::io::{stderr, stdout, Stdout};
use std::panic;
use color_eyre::eyre;
use editor::Editor;
use color_eyre::eyre::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::{ExecutableCommand, execute};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
pub use terminal::Terminal;
pub use editor::Position;
pub use editor::SearchDirection;
pub use document::Document;
pub use filetype::{FileType, HighlightingOptions};
pub use row::Row;

use crate::app::App;

fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    let mut terminal = init_terminal()?;

    install_hooks()?;

    // Create app and run it
    let mut app = App::default();
    app.run(&mut terminal)?;

    // Restore terminal
    restore()?;
    execute!(terminal.backend_mut())?;
    terminal.show_cursor()?;

    Ok(())
}

fn init_terminal() -> Result<ratatui::Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = ratatui::Terminal::new(backend)?;

    Ok(terminal)
}

fn restore() -> Result<()> {
    disable_raw_mode()?;
    stderr().execute(LeaveAlternateScreen)?;
    stderr().execute(DisableMouseCapture)?;

    Ok(())
}

fn main_old() -> Result<()> {
    install_hooks_old()?;

    Editor::default().run();

    Ok(())
}

fn install_hooks() -> Result<()> {
    let hook_builder = color_eyre::config::HookBuilder::default();
    let (panic_hook, eyre_hook) = hook_builder.into_hooks();

    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore().unwrap();
        panic_hook(panic_info);
    }));

    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |error| {
        restore().unwrap();
        eyre_hook(error)
    }))?;

    Ok(())
}

fn install_hooks_old() -> Result<()> {
    let hook_builder = color_eyre::config::HookBuilder::default();
    let (panic_hook, eyre_hook) = hook_builder.into_hooks();

    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        Terminal::restore();
        panic_hook(panic_info);
    }));

    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |error| {
        Terminal::restore();
        eyre_hook(error)
    }))?;

    Ok(())
}