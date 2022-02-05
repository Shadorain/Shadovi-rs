use crate::Document;
use crate::Row;
use crate::Terminal;

use std::env;
use termion::event::Key;

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
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
        let document = if args.len() > 1 {
            let file_name = &args[1];
            Document::open(&file_name).unwrap_or_default()
        } else {
            Document::default()
        };
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
        }
    }

    fn draw_welcome_msg (&self) {
        let mut welcome_msg = format!("{} -- v{}", crate::NAME, crate::VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_msg.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_msg = format!("~{}{}", spaces, welcome_msg);
        welcome_msg.truncate(width);
        println!("{}\r", welcome_msg);
        // println!("{}", crate::DESCRIPTION);
        // println!("{}", crate::REPOSITORY);
    }

    pub fn draw_row (&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r", row)
    }

    fn draw_rows (&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0 .. height - 1 {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_msg();
            } else { println!("~\r"); }
        }
    }

    fn process_keypress (&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Up | Key::Char('k')
                | Key::Down | Key::Char('j')
                | Key::Left | Key::Char('h')
                | Key::Right | Key::Char('l')
                | Key::PageUp
                | Key::PageDown
                | Key::End
                | Key::Home
                => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn move_cursor (&mut self, key: Key) {
        let Position { mut x, mut y } = self.cursor_position;
        let size = self.terminal.size();
        let height = size.height.saturating_sub(1) as usize;
        let width = size.width.saturating_sub(1) as usize;
        match key {
            Key::Up    | Key::Char('k') => y = y.saturating_sub(1),
            Key::Down  | Key::Char('j') => {
                if y < height {
                    y = y.saturating_add(1);
                }
            },
            Key::Left  | Key::Char('h') => x = x.saturating_sub(1),
            Key::Right | Key::Char('l') => {
                if x < width {
                    x = x.saturating_add(1);
                }
            },
            Key::PageUp   => y = 0,
            Key::PageDown => y = height,
            Key::Home     => x = 0,
            Key::End      => x = width,
            _ => (),
        }
        self.cursor_position = Position { x, y }
    }

    fn scroll (&mut self) {
        let Position { x, y } = self.cursor_position;
    }

    fn refresh_screen (&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            Terminal::cursor_position(&self.cursor_position);
        }
        Terminal::cursor_show();
        Terminal::flush()
    }
}

fn die (e: &std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
