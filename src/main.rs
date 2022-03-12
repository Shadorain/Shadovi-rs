#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(clippy::missing_docs_in_private_items, clippy::implicit_return,
    clippy::shadow_reuse, clippy::print_stdout, clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else, clippy::missing_errors_doc, clippy::cast_possible_truncation,
    clippy::blanket_clippy_restriction_lints, clippy::must_use_candidate
)]

mod document;
mod editor;
mod filetype;
mod highlighting;
mod row;
mod terminal;

use editor::Editor;
pub use document::Document;
pub use editor::{Position, SearchDirection};
pub use filetype::{FileType, HighlightingOptions};
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
