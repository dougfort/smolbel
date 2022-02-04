use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Error};

pub mod object;
pub use object::Object;

pub mod parser;
pub use parser::parse;

mod primatives;
use primatives::{load_primatives, PrimFunc};

mod loader;
pub use loader::load_source;

pub struct Bel {
    pub globals: HashMap<String, Object>,
    primatives: HashMap<String, PrimFunc>,
    function_names: HashSet<String>,
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
            function_names: HashSet::new(),
        }
    }

    pub fn eval(
        &mut self,
        locals: &HashMap<String, Object>,
        exp: &Object,
    ) -> Result<Object, Error> {
        let output = match exp {
            Object::Symbol(name) => self.get_bound_object(locals, name)?,
            Object::Pair(pair) => self.eval_pair(locals, pair)?,
            Object::Char(_c) => {
                return Err(anyhow!("Object::Char not implemented"));
            }
            Object::Stream => {
                return Err(anyhow!("Object::Stream not implemented"));
            }
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

    fn eval_pair(
        &mut self,
        locals: &HashMap<String, Object>,
        pair: &(Object, Object),
    ) -> Result<Object, Error> {
        let (car, cdr) = &*pair;
        if let Object::Symbol(name) = car {
            match name.as_ref() {
                "set" => self.set(&cdr),
                "def" => self.def(&cdr),                 
                "mac" => self.mac(&cdr), 
                n if self.primatives.contains_key(n) => {
                    let evaluated_list = self.evaluate_list(locals, &cdr)?;
                    self.primatives[n](&evaluated_list)
                },
                n if self.function_names.contains(n) => {
                    let evaluated_list = self.evaluate_list(locals, &cdr)?;
                    self.apply_function(locals, n, &evaluated_list)
                }
                _ => Err(anyhow!("eval_pair: not implemented: {}", name))
            }
        } else {
            self.evaluate_list(locals, &cdr)
        }
    }

    fn set(&mut self, args: &Object) -> Result<Object, Error> {
        Err(anyhow!("set not implemented"))
    }

    fn def(&mut self, args: &Object) -> Result<Object, Error> {
        Err(anyhow!("def not implemented"))
    }

    fn mac(&mut self, args: &Object) -> Result<Object, Error> {
        Err(anyhow!("mac not implemented"))
    }

    fn apply_function(&mut self, locals: &HashMap<String, Object>, name: &str, args: &Object) -> Result<Object, Error> {
        Err(anyhow!("apply_function not implemented"))
    }

    fn evaluate_list(&mut self, locals: &HashMap<String, Object>, list: &Object) -> Result<Object, Error> {
        Err(anyhow!("evaluate_list not implemented"))
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
        let locals: HashMap<String, Object> = HashMap::new();
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
