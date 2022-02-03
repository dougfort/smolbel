use anyhow::Error;

pub mod object;
use object::Object;

pub mod parser;
pub use parser::parse;

#[derive(Debug)]
pub struct Bel {}

impl Bel {
    pub fn new() -> Self {
        Bel {}
    }

    pub fn eval(&mut self, _exp: &Object) -> Result<Object, Error> {
        Ok(object::nil())
    }
}

impl Default for Bel {
    fn default() -> Self {
        Self::new()
    }
}
