use crate::Position;
use std::io::{self, stdout, Write, ErrorKind};

use crossterm::event::{KeyCode, KeyEvent};
use crossterm::event::read;
use crossterm::event;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::ClearType;
use crossterm::style::Color;
use std::fs::read_to_string;


pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },

        })
    }
    pub fn size(&self) -> &Size {
        &self.size
    }
    pub fn clear_screen() {
        print!("{}", crossterm::terminal::Clear(ClearType::All));
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(position: &Position) {
        let Position { mut x, x_word_index, mut y } = position;
        let x = x as u16;
        let y = y as u16;
        print!("{}", crossterm::cursor::MoveTo(x, y));
    }
    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }

    pub fn read_key() -> Result<KeyEvent, std::io::Error> {
        if let event::Event::Key(key) = read()? {
            return Ok(key);
        }
        Err(std::io::Error::new(ErrorKind::Other, "oh no!"))
    }
    pub fn cursor_hide() {
        print!("{}", crossterm::cursor::Hide);
    }
    pub fn cursor_show() {
        print!("{}", crossterm::cursor::Show);
    }
    pub fn clear_current_line() {
        print!("{}", crossterm::terminal::Clear(ClearType::CurrentLine));
    }
    pub fn set_bg_color(color: Color) {
        print!("{}", crossterm::style::SetBackgroundColor(color));
    }

    pub fn set_fg_color(color: Color) {
        print!("{}", crossterm::style::SetForegroundColor(color));
    }

    pub fn reset_color() {
        print!("{}", crossterm::style::ResetColor);
    }
}
