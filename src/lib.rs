#[macro_use]
pub mod object;
pub use object::Object;

pub mod parser;
pub use parser::parse;

pub mod primatives;

pub mod functions;

pub mod loader;

pub mod list;

pub mod eval;
