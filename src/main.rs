#![warn(clippy::all, clippy::pedantic)]

mod editor;
use editor::Editor;

mod terminal;

fn main () {
    Editor::default().run();
}
