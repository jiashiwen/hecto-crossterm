use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::time::Duration;
use std::time::Instant;
use crossterm::style::Color;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::disable_raw_mode;


const STATUS_FG_COLOR: Color = Color::Rgb { r: 63, g: 63, b: 63 };
const STATUS_BG_COLOR: Color = Color::Rgb {
    r: 135,
    g: 206,
    b: 235,
};


const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub x_word_index: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
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

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status =
            String::from("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");

        let document = if let Some(file_name) = args.get(1) {
            let doc = Document::open(file_name);
            if let Ok(doc) = doc {
                doc
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
            highlighted_word: None,
        }
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.document.highlight(
                &self.highlighted_word,
                Some(
                    self.offset
                        .y
                        .saturating_add(self.terminal.size().height as usize),
                ),
            );
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                x_word_index: 0,
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
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }
    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;
        let query = self
            .prompt(
                "Search (ESC to cancel, Arrows to navigate): ",
                |editor, key, query| {
                    let mut moved = false;
                    match key {
                        KeyCode::Right | KeyCode::Down => {
                            direction = SearchDirection::Forward;
                            editor.move_cursor(KeyCode::Right);
                            moved = true;
                        }
                        KeyCode::Left | KeyCode::Up => direction = SearchDirection::Backward,
                        _ => direction = SearchDirection::Forward,
                    }
                    if let Some(position) =
                    editor
                        .document
                        .find(&query, &editor.cursor_position, direction)
                    {
                        editor.cursor_position = position;
                        editor.scroll();
                    } else if moved {
                        editor.move_cursor(KeyCode::Left);
                    }
                    editor.highlighted_word = Some(query.to_string());
                },
            )
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        self.highlighted_word = None;
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            KeyEvent {
                code: KeyCode::Char('q'), modifiers: KeyModifiers::CONTROL
            } => {
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                        self.quit_times
                    ));
                    self.quit_times -= 1;
                    return Ok(());
                }
                disable_raw_mode();
                self.should_quit = true
            }
            // Key::Ctrl('s') => self.save(),
            KeyEvent {
                code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL
            } => self.save(),
            // Key::Ctrl('f') => self.search(),
            KeyEvent {
                code: KeyCode::Char('f'), modifiers: KeyModifiers::CONTROL
            } => self.search(),
            KeyEvent {
                code: KeyCode::Char(c), ..
            } => {
                self.document.insert(&mut self.cursor_position, c);
                self.move_cursor(KeyCode::Right);
            }
            KeyEvent {
                code: KeyCode::Enter, ..
            } => {
                self.document.insert_newline(&self.cursor_position);
                self.cursor_position.y = self.cursor_position.y.saturating_add(1);
                self.cursor_position.x = 0;
                self.cursor_position.x_word_index = 0;
            }
            KeyEvent {
                code: KeyCode::Delete, ..
            } => self.document.delete(&self.cursor_position),

            KeyEvent {
                code: KeyCode::Backspace, ..
            } => {
                if self.cursor_position.x_word_index > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(KeyCode::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            KeyEvent {
                code: KeyCode::Up, ..
            }
            | KeyEvent {
                code: KeyCode::Down, ..
            }
            | KeyEvent {
                code: KeyCode::Left, ..
            }
            | KeyEvent {
                code: KeyCode::Right, ..
            }
            | KeyEvent {
                code: KeyCode::PageUp, ..
            }
            | KeyEvent {
                code: KeyCode::PageDown, ..
            }
            | KeyEvent {
                code: KeyCode::End, ..
            }
            | KeyEvent {
                code: KeyCode::Home, ..
            } => self.move_cursor(pressed_key.code),
            _ => (),
        }
        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }
    fn scroll(&mut self) {
        let Position { x, x_word_index, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
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
    fn move_cursor(&mut self, key: KeyCode) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x_word_index, mut x } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            // row.len()
            row.width()
        } else {
            0
        };
        match key {
            KeyCode::Up => {
                y = y.saturating_sub(1);
                //计算x值,更新word_index
                // if y > 0 {
                if x_word_index > self.document.rows[y]
                    .word_width_index.len() {
                    x_word_index = self.document.rows[y]
                        .word_width_index.len();
                }
                let mut x_cursor = 0;
                for word_len in self.document.rows[y]
                    .word_width_index.clone().iter().take(x_word_index) {
                    x_cursor = x_cursor + word_len;
                }
                x = x_cursor;
                // }
            }
            KeyCode::Down => {
                //计算x值,更新word_index
                if y >= 0 && height > 0 && y < height - 1 {
                    y = y.saturating_add(1);
                    if y < self.document.rows.len() {
                        if x_word_index > self.document.rows[y]
                            .word_width_index.len() {
                            x_word_index = self.document.rows[y]
                                .word_width_index.len();
                        }
                        let mut x_cursor = 0;
                        for word_len in self.document.rows[y]
                            .word_width_index.clone().iter().take(x_word_index) {
                            x_cursor = x_cursor + word_len;
                        }
                        x = x_cursor;
                    }
                }
                log::info!("y: {}",y);
            }
            KeyCode::Left => {
                // if x > 0 {
                if x_word_index > 0 && y < height {
                    x_word_index -= 1;
                    let mut x_cursor = 0;
                    for word_len in self.document.rows[y]
                        .word_width_index.clone().iter().take(x_word_index) {
                        x_cursor = x_cursor + word_len;
                    }
                    x = x_cursor;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        // x = row.len();
                        let mut x_cursor = 0;
                        for word_len in self.document.rows[y]
                            .word_width_index.clone().iter() {
                            x_cursor = x_cursor + word_len;
                        }
                        x_word_index = self.document.rows[y]
                            .word_width_index
                            .len();
                        x = x_cursor;
                    } else {
                        x = 0;
                    }
                }
            }
            KeyCode::Right => {
                // if x < width {
                if y < height && x_word_index < self.document.rows[y].word_width_index.len() {
                    x_word_index += 1;
                    let mut x_cursor = 0;
                    for word_len in self.document.rows[y]
                        .word_width_index.clone().iter().take(x_word_index) {
                        x_cursor = x_cursor + word_len;
                    }

                    x = x_cursor;
                    // // x += 1;
                    // let mut index = x.saturating_add(1);
                    // let str = &self.document.rows[y].string;
                    // while index < str.len()
                    //     && !str.is_char_boundary(index) {
                    //     index += 1;
                    // }
                    //
                    // if index - x > 1 {
                    //     x += 2;
                    // } else {
                    //     x += 1;
                    // }
                } else if y < height - 1 {
                    y += 1;
                    x = 0;
                    x_word_index = 0;
                }
            }
            KeyCode::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            KeyCode::Home => x = 0,
            KeyCode::End => {
                let mut x_cursor_position: usize = 0;
                for word_space in self.document.rows[y].word_width_index.clone() {
                    x_cursor_position = x_cursor_position + word_space;
                }
                x = x_cursor_position
            }
            _ => (),
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        // let mut x_cursor_position: usize = 0;
        // for word_space in self.document.rows[y].word_width_index.clone() {
        //     x_cursor_position = x_cursor_position + word_space;
        // }
        // if x > x_cursor_position {
        //     x = x_cursor_position;
        // }
        // if x > width {
        //     x = width;
        // }

        self.cursor_position = Position { x, x_word_index, y }
    }
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hecto editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
            let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }
    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row)
    }
    #[allow(clippy::integer_division, clippy::integer_arithmetic)]
    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };

        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.len(),
            modified_indicator
        );

        let line_indicator = format!(
            "{} | {}/{}",
            self.document.file_type(),
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        #[allow(clippy::integer_arithmetic)]
            let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_color();
    }
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }
    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
        where
            C: FnMut(&mut Self, KeyCode, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let keyevent = Terminal::read_key()?;
            let key = keyevent.code;
            match key {
                KeyCode::Backspace => result.truncate(result.len().saturating_sub(1)),
                // KeyCode::Char('\n') => break,
                KeyCode::Enter => break,
                KeyCode::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                KeyCode::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }
}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!(e);
}
