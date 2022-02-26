use crate::{Document, Row, Terminal};

use std::env;
use std::intrinsics::caller_location;
use std::time::{Duration, Instant};
use termion::color;
use termion::event::Key;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);

const QUIT_THRESH: u8 = 2;

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from (message: String) -> Self {
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
    quit_thresh: u8,
}

impl Editor {
    pub fn run (&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() { die(&error); }
            if self.should_quit { break }
            if let Err(error) = self.process_keypress() { die(&error); }
        }
    }

    pub fn default () -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("Help: C-s to save | C-q to quit | C-f to search");
        let document = if let Some(file_name) = args.get(1) {
            let doc = Document::open(file_name);
            if let Ok(doc) = doc {
                doc
            } else {
                initial_status = format!("Err: Couldn't open file: {}", file_name);
                Document::default()
            }
        } else { Document::default() };
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
            status_message: StatusMessage::from(initial_status),
            quit_thresh: QUIT_THRESH,
        }
    }

    fn process_keypress (&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => {
                if self.quit_thresh > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(format!("[WARN]: File has unsaved changes"));
                    self.quit_thresh -= 1;
                    return Ok(());
                }
                self.should_quit = true
            }
            Key::Ctrl('s') => self.save(),
            Key::Ctrl('f') => {
                if let Some(query) = self.prompt("Search: ", |editor, _, query| {
                    if let Some(pos) = self.document.find(&query) {
                        editor.cursor_position = pos;
                        editor.scroll();
                    }
                }).unwrap_or(None) {
                    if let Some(pos) = self.document.find(&query[..]) {
                        self.cursor_position = pos;
                    } else {
                        self.status_message = StatusMessage::from(format!("Not found: {}", query));
                    }
                }
            },
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            },
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
            },
            Key::Up // | Key::Char('k')
                | Key::Down  //| Key::Char('j')
                | Key::Left  //| Key::Char('h')
                | Key::Right //| Key::Char('l')
                | Key::PageUp
                | Key::PageDown
                | Key::End
                | Key::Home
                => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        if self.quit_thresh < QUIT_THRESH {
            self.quit_thresh = QUIT_THRESH;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    fn save (&mut self) {
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
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        }
    }

    fn prompt <C>(&mut self, prompt: &str, callback: C) -> Result<Option<String>, std::io::Error>
        where C: Fn(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => result.truncate(result.len().saturating_sub(1)),
                Key::Char('\n') => break,
                Key::Char(c) => { if !c.is_control() { result.push(c); } },
                Key::Esc => { result.truncate(0); break; },
                _ => (),
            }
            callback(self, key, &result);
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() { return Ok(None); }
        Ok(Some(result))
    }

    fn draw_welcome_msg (&self) {
        let mut welcome_msg = format!("{} -- v{}", crate::NAME, crate::VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_msg.len();
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_msg = format!("~{}{}", spaces, welcome_msg);
        welcome_msg.truncate(width);
        println!("{}\r", welcome_msg);
        // println!("{}", crate::DESCRIPTION);
        // println!("{}", crate::REPOSITORY);
    }

    fn draw_status_bar (&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() { "*" } else { "" };
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!("{}{} - {}", modified_indicator, file_name, self.document.len());
        let line_indicator = format!(
            "{}/{}",
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
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar (&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    pub fn draw_row (&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    #[allow(clippy::integer_arithmetic, clippy::integer_division)]
    fn draw_rows (&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0 .. height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(self.offset.y.saturating_add(terminal_row as usize)) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_msg();
            } else { println!("~\r"); }
        }
    }

    #[allow(clippy::integer_arithmetic)]
    fn move_cursor (&mut self, key: Key) {
        let Position { mut x, mut y } = self.cursor_position;
        let terminal_height = self.terminal.size().height as usize;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) { row.len() } else { 0 };
        match key {
            Key::Up    /*| Key::Char('k')*/ => y = y.saturating_sub(1),
            Key::Down  /*| Key::Char('j')*/ => {
                if y < height {
                    y = y.saturating_add(1);
                }
            },
            Key::Left  /*| Key::Char('h')*/ => {
                if x > 0 { x -= 1; }
                else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else { x = 0; }
                }
            },
            Key::Right /*| Key::Char('l')*/ => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            },
            Key::PageUp   => {
                y = if y > terminal_height { y.saturating_sub(terminal_height) } else { 0 }
            },
            Key::PageDown => y = {
                if y.saturating_add(terminal_height) < height { y.saturating_add(terminal_height) }
                else { height }
            },
            Key::Home     => x = 0,
            Key::End      => x = width,
            _ => (),
        }
        /* Update positioning after keypress */
        width = if let Some(row) = self.document.row(y) { row.len() } else { 0 };
        if x > width { x = width; }
        self.cursor_position = Position { x, y }
    }

    fn scroll (&mut self) {
        let Position { x, y } = self.cursor_position;
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

    fn refresh_screen (&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
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
}

fn die (e: &std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
