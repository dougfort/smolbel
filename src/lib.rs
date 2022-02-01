use anyhow::{anyhow, Error};

pub mod object;
use object::Object;

#[derive(Default, Debug)]
pub struct Bel {

}

impl Bel {

    pub fn new() -> Self {
        Bel {}
    }

    pub fn eval(&mut self, text: &str) -> Result<Object, Error> {
        text.chars().for_each(|c| {
            match c {
                _ => {}
            }
        });

        Ok(Object::nil())
    }

}
