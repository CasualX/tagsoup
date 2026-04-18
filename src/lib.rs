use std::collections::HashMap;
use std::fmt;

mod document;
mod element;
mod errors;
mod map;
mod nodes;
mod tagsoup;
mod selector;
mod span;
mod utils;
mod visit;

pub use document::*;
pub use element::*;
pub use errors::*;
pub use map::*;
pub use nodes::*;
pub use span::*;
pub use utils::*;
pub use visit::*;
