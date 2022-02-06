use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Error};

#[macro_use]
pub mod object;
pub use object::Object;

pub mod parser;
pub use parser::parse;

mod primatives;
use primatives::{load_primatives, PrimFunc};

mod loader;
pub use loader::load_source;

pub type ObjectMap = HashMap<String, Object>;

pub struct Bel {
    pub globals: ObjectMap,
    primatives: HashMap<String, PrimFunc>,
    function_names: HashSet<String>,
    macro_names: HashSet<String>,
}

impl Bel {
    pub fn new() -> Self {
        Bel {
            // some Symbols bind to themselves
            globals: HashMap::from([
                ("nil".to_string(), nil!()),
                ("t".to_string(), object::symbol("t")),
                ("o".to_string(), object::symbol("o")),
                ("apply".to_string(), object::symbol("apply")),
            ]),
            primatives: load_primatives(),
            function_names: HashSet::new(),
            macro_names: HashSet::new(),
        }
    }

    pub fn eval(&mut self, exp: &Object) -> Result<Object, Error> {
        let output = match exp {
            Object::Symbol(name) => self.get_bound_object(name)?,
            Object::Pair(pair) => self.eval_pair(pair)?,
            Object::Char(_c) => {
                return Err(anyhow!("Object::Char not implemented"));
            }
            Object::Stream => {
                return Err(anyhow!("Object::Stream not implemented"));
            }
        };

        Ok(output)
    }

    fn get_bound_object(&self, name: &str) -> Result<Object, Error> {
        match self.globals.get(name) {
            Some(obj) => Ok(obj.clone()),
            None => Err(anyhow!("unbound symbol: {:?}", name)),
        }
    }

    fn eval_pair(&mut self, pair: &(Object, Object)) -> Result<Object, Error> {
        let (car, cdr) = &*pair;
        if let Object::Symbol(name) = car {
            match name.as_ref() {
                "set" => self.set(cdr),
                "def" => self.def(cdr),
                "mac" => self.mac(cdr),
                "quote" => quote(cdr),
                n if self.primatives.contains_key(n) => {
                    let evaluated_list = self.evaluate_list(cdr)?;
                    self.primatives[n](&evaluated_list)
                }
                n if self.function_names.contains(n) => {
                    let evaluated_list = self.evaluate_list(cdr)?;
                    self.apply_function(n, &evaluated_list)
                }
                n if self.macro_names.contains(n) => {
                    let evaluated_list = self.evaluate_list(cdr)?;
                    self.apply_macro(n, &evaluated_list)
                }
                _ => self.evaluate_list(cdr),
            }
        } else {
            self.evaluate_list(cdr)
        }
    }

    fn set(&mut self, args: &Object) -> Result<Object, Error> {
        let list = args.to_vec()?;
        for i in 0..list.len() - 1 {
            if let Object::Symbol(key) = list[i].clone() {
                self.globals.insert(key, list[i + 1].clone());
            } else {
                return Err(anyhow!(
                    "invalid object: expected: {} found: {}",
                    "symbol".to_string(),
                    list[i].t(),
                ));
            }
        }
        // append nil if the final arg isn't present
        // an odd number of entries
        // means the last value is unspecified
        if list.len() % 2 == 1 {
            let i = list.len() - 1;
            if let Object::Symbol(key) = list[i].clone() {
                self.globals.insert(key, nil!());
            } else {
                return Err(anyhow!(
                    "invalid object: expected: {} found: {}",
                    "symbol".to_string(),
                    list[i].t(),
                ));
            }
        }
        Ok(nil!())
    }
    // When you see
    //  (def n p e)
    // treat it as an abbreviation for
    //  (set n (lit clo nil p e))
    fn def(&mut self, args: &Object) -> Result<Object, Error> {
        let (name, body) = define_closure(args)?;
        let fn_def = object::from_vec(vec![symbol!(name), body])?;
        self.function_names.insert(name);
        self.set(&fn_def)
    }

    // when you see
    //  (mac n p e)
    // treat it as an abbreviation for
    //  (set n (lit mac (lit clo nil p e)))
    fn mac(&mut self, args: &Object) -> Result<Object, Error> {
        let (name, body) = define_closure(args)?;
        let mac_body = object::from_vec(vec![object::symbol("lit"), object::symbol("mac"), body])?;
        let mac_def = object::from_vec(vec![symbol!(name), mac_body])?;
        self.macro_names.insert(name);
        self.set(&mac_def)
    }

    fn apply_function(&mut self, _name: &str, _args: &Object) -> Result<Object, Error> {
        Err(anyhow!("apply_function not implemented"))
    }

    fn apply_macro(&mut self, _name: &str, _args: &Object) -> Result<Object, Error> {
        Err(anyhow!("apply_function not implemented"))
    }

    fn evaluate_list(&mut self, list: &Object) -> Result<Object, Error> {
        let mut accum: Object = nil!();
        let mut list = list.clone();

        while !list.is_nil() {
            if let Object::Pair(pair) = list {
                let (car, cdr) = *pair.clone();
                let obj = self.eval(&car)?;
                accum = object::join(obj, accum)?;
                list = cdr;
            } else {
                return Err(anyhow!("expected Pair: {:?}", list));
            }
        }

        Ok(accum)
    }
}

impl Default for Bel {
    fn default() -> Self {
        Self::new()
    }
}

// The quote operator returns its argument without evaluating it.
// Its purpose is to prevent evaluation.
fn quote(params: &Object) -> Result<Object, Error> {
    let params = params.clone();
    if let Object::Pair(pair) = params {
        let pair = *pair;
        let (car, cdr) = pair;
        if cdr.is_nil() {
            Ok(car)
        } else {
            Err(anyhow!(
                "quote expecting single element list; found {:?}",
                cdr
            ))
        }
    } else {
        Err(anyhow!("quote: expecting pair found: {:?}", params))
    }
}

fn define_closure(list: &Object) -> Result<(String, Object), Error> {
    let args = list.to_vec()?;
    if args.len() == 3 {
        if let Object::Symbol(name) = args[0].clone() {
            let p = args[1].clone();

            let e = args[2].clone();
            let body = object::from_vec(vec![
                object::symbol("lit"),
                object::symbol("clo"),
                nil!(),
                p,
                e,
            ])?;
            Ok((name, body))
        } else {
            Err(anyhow!("invalid def name {:?}", args))
        }
    } else {
        Err(anyhow!("invalid def {:?}", args))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_object() -> Result<(), Error> {
        let mut bel = Bel::new();
        let exp = parse("t")?;
        let obj = bel.eval(&exp)?;
        assert_eq!(exp, obj);
        Ok(())
    }

    #[test]
    fn can_set_object() -> Result<(), Error> {
        let mut bel = Bel::new();
        let exp = parse("(set a b)")?;
        let obj = bel.eval(&exp)?;
        assert!(obj.is_nil());

        let exp = parse("a")?;
        let obj = bel.eval(&exp)?;
        assert_eq!(obj, object::symbol("b"));
        Ok(())
    }

    #[test]
    fn can_set_multiple() -> Result<(), Error> {
        let mut bel = Bel::new();

        let parse_obj = parse("(set a b c d e f)")?;
        let obj = bel.eval(&parse_obj)?;
        assert!(obj.is_nil());

        for (key, val) in &[
            ("a", "b".to_string()),
            ("c", "d".to_string()),
            ("e", "f".to_string()),
        ] {
            let parse_obj = parse(key)?;
            let obj = bel.eval(&parse_obj)?;
            if let Object::Symbol(s) = obj {
                assert_eq!(&s, val);
            } else {
                panic!("unexpected object {:?}", obj);
            }
        }

        Ok(())
    }

    #[test]
    fn can_set_multiple_with_default() -> Result<(), Error> {
        let mut bel = Bel::new();

        let parse_obj = parse("(set a b c d e)")?;
        let obj = bel.eval(&parse_obj)?;
        assert!(obj.is_nil());

        for (key, val) in &[
            ("a", "b".to_string()),
            ("c", "d".to_string()),
            ("e", "nil".to_string()),
        ] {
            let parse_obj = parse(key)?;
            let obj = bel.eval(&parse_obj)?;
            if let Object::Symbol(s) = obj {
                assert_eq!(&s, val);
            } else {
                panic!("unexpected object {:?}", obj);
            }
        }

        Ok(())
    }

    #[test]
    fn can_quote_object() -> Result<(), Error> {
        let mut bel = Bel::new();

        let parse_obj = parse("(set a b)")?;
        let obj = bel.eval(&parse_obj)?;
        assert!(obj.is_nil());

        let parse_obj = parse("(quote a)")?;
        let obj = bel.eval(&parse_obj)?;
        if let Object::Symbol(s) = obj {
            assert_eq!(s, "a");
        } else {
            panic!("unexpected object {:?}", obj);
        }

        let parse_obj = parse("(quote ( x ))")?;
        let obj = bel.eval(&parse_obj)?;
        assert_eq!(obj, object::pair(object::symbol("x"), nil!()));

        Ok(())
    }

    #[test]
    fn can_def_a_function() -> Result<(), Error> {
        let mut bel = Bel::new();

        let parse_obj = parse(
            r#"(def xnox (x)
                (id x nil))
          "#,
        )?;
        let obj = bel.eval(&parse_obj)?;
        assert!(obj.is_nil());

        let parse_obj = parse("(xnox nil)")?;
        let obj = bel.eval(&parse_obj)?;
        assert!(obj.is_true());

        let parse_obj = parse("(xnox `a)")?;
        let obj = bel.eval(&parse_obj)?;
        assert!(obj.is_nil(), "{:?}", obj);

        Ok(())
    }

    #[test]
    fn can_load_function() -> Result<(), Error> {
        Ok(())
    }
}
