use std::collections::HashMap;

use anyhow::{anyhow, Error};

pub mod object;
pub use object::Object;

pub mod parser;
pub use parser::parse;

mod primatives;
use primatives::{load_primatives, PrimFunc};

mod loader;
pub use loader::load_commands;

pub struct Bel {
    pub globals: HashMap<String, Object>,
    primatives: HashMap<String, PrimFunc>,
}

impl Bel {
    pub fn new() -> Self {
        Bel {
            // some Symbols bind to themselves
            globals: HashMap::from([
                ("nil".to_string(), object::symbol("nil")),
                ("t".to_string(), object::symbol("t")),
                ("o".to_string(), object::symbol("o")),
                ("apply".to_string(), object::symbol("apply")),
            ]),
            primatives: load_primatives(),
        }
    }

    pub fn eval(
        &mut self,
        locals: &HashMap<String, Object>, 
        exp: &Object,
    ) -> Result<Object, Error> {

        let output = match exp {
            Object::Symbol(name) => self.get_bound_object(locals, name)?,
            _ => object::nil(),
        };

        Ok(output)
    }

    fn get_bound_object(
        &self,
        locals: &HashMap<String, Object>,
        name: &str,
    ) -> Result<Object, Error> {
        // look first in locals, then in globals
        match locals.get(name) {
            Some(obj) => Ok(obj.clone()),
            None => match self.globals.get(name) {
                Some(obj) => Ok(obj.clone()),
                None => Err(anyhow!("unbound symbol: {:?}", name)),
            },
        }
    }

}

impl Default for Bel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_object() -> Result<(), Error> {
        let mut bel = Bel::new();
        let 
        locals: HashMap<String, Object> = HashMap::new();
        let exp = parse("t")?;
        let obj = bel.eval(&locals, &exp)?;
        assert_eq!(exp, obj);
        Ok(())
    }

    #[test]
    fn can_load_function() -> Result<(), Error> {
        Ok(())
    }
}
