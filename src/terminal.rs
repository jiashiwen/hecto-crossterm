use crate::Position;
use std::io::{self, stdout, Write, ErrorKind};
// use termion::color;
// use termion::event::Key;
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::event::read;
use crossterm::event;
// use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
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
    // _stdout: RawTerminal<std::io::Stdout>,
    // _stdout: std::io::Stdout,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            // _stdout: stdout().into_raw_mode()?,
        })
    }
    pub fn size(&self) -> &Size {
        &self.size
    }
    pub fn clear_screen() {
        // print!("{}", termion::clear::All);
        print!("{}", crossterm::terminal::Clear(ClearType::All));
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(position: &Position) {
        let Position { mut x, mut y } = position;
        // x = x.saturating_add(1);
        // y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        // print!("{}", termion::cursor::Goto(x, y));
        print!("{}", crossterm::cursor::MoveTo(x, y));
    }
    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }
    // pub fn read_key() -> Result<KeyCode, std::io::Error> {
    pub fn read_key() -> Result<KeyEvent, std::io::Error> {
        // enable_raw_mode();
        if let event::Event::Key(key) = read()? {
            return Ok(key);
        }
        Err(std::io::Error::new(ErrorKind::Other, "oh no!"))
        // loop {
        //     if let Some(key) = io::stdin().lock().keys().next() {
        //         return key;
        //     }
        // }
    }
    pub fn cursor_hide() {
        // print!("{}", termion::cursor::Hide);
        print!("{}", crossterm::cursor::Hide);
    }
    pub fn cursor_show() {
        // print!("{}", termion::cursor::Show);
        print!("{}", crossterm::cursor::Show);
    }
    pub fn clear_current_line() {
        // print!("{}", termion::clear::CurrentLine);
        print!("{}", crossterm::terminal::Clear(ClearType::CurrentLine));
    }
    pub fn set_bg_color(color: Color) {
        // print!("{}", color::Bg(color));
        print!("{}", crossterm::style::SetBackgroundColor(color));
    }

    pub fn set_fg_color(color: Color) {
        // print!("{}", color::Fg(color));
        print!("{}", crossterm::style::SetForegroundColor(color));
    }

    pub fn reset_color() {
        print!("{}", crossterm::style::ResetColor);
    }
    // pub fn reset_bg_color() {
    //     print!("{}", color::Bg(color::Reset));
    //     // print!("{}", crossterm::style::ResetColor);
    // }
    // pub fn reset_fg_color() {
    //     print!("{}", color::Fg(color::Reset));
    // }
}
