use anyhow::Error;

pub mod object;
use object::Object;

pub mod parser;

#[derive(Debug)]
pub struct Bel {}

impl Bel {
    pub fn new() -> Self {
        Bel {}
    }

    pub fn eval(&mut self, _text: &str) -> Result<Object, Error> {
        Ok(object::nil())
    }
}

impl Default for Bel {
    fn default() -> Self {
        Self::new()
    }
}
