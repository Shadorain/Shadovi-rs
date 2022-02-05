#![warn(clippy::all, clippy::pedantic)]

mod document;
mod editor;
mod row;
mod terminal;

use editor::Editor;
pub use document::Document;
pub use editor::Position;
pub use row::Row;
pub use terminal::Terminal;

pub const NAME: &str = "ShadoVi"/* env!("CARGO_PKG_NAME") */;
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

fn main () {
    Editor::default().run();
}
