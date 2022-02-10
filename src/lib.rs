use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Error};
use log::debug;

#[macro_use]
pub mod object;
pub use object::Object;

pub mod parser;
pub use parser::parse;

mod primatives;
use primatives::{load_primatives, PrimFunc};

mod loader;
pub use loader::load_source;

mod list;
pub use list::List;

pub type ObjectMap = HashMap<String, Object>;

pub struct Bel {
    pub globals: ObjectMap,
    pub primatives: HashMap<String, PrimFunc>,
    pub function_names: HashSet<String>,
    pub macro_names: HashSet<String>,
}

impl Bel {
    pub fn new() -> Self {
        Bel {
            // some Symbols bind to themselves
            globals: HashMap::from([
                ("nil".to_string(), nil!()),
                ("t".to_string(), symbol!("t")),
                ("o".to_string(), symbol!("o")),
                ("apply".to_string(), symbol!("apply")),
            ]),
            primatives: load_primatives(),
            function_names: HashSet::new(),
            macro_names: HashSet::new(),
        }
    }

    pub fn eval(&mut self, exp: &Object) -> Result<Object, Error> {
        debug!("eval: exp = {}", exp);
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
                _ => Err(anyhow!("unknown symbol: {}", name)),
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
        let mac_body = object::from_vec(vec![symbol!("lit"), symbol!("mac"), body])?;
        let mac_def = object::from_vec(vec![symbol!(name), mac_body])?;
        self.macro_names.insert(name);
        self.set(&mac_def)
    }

    fn apply_function(&mut self, f_name: &str, args: &Object) -> Result<Object, Error> {
        debug!("apply_function: f_name= {}, args= {}", f_name, args);

        let f_obj = if let Some(f) = self.globals.get(f_name) {
            f
        } else {
            return Err(anyhow!("unknown function {}", f_name));
        };

        let mut list = List::new(f_obj);

        // we expect the function to contain 5 items
        // starting with the symbols lit clo nil
        for &name in &["lit", "clo", "nil"] {
            match list.step()? {
                Some(obj) => {
                    if let Object::Symbol(symbol_name) = obj.clone() {
                        if symbol_name != name {
                            return Err(anyhow!(
                                "apply_function: unexpected symbol: {:?}; expected {}",
                                obj,
                                name
                            ));
                        }
                    } else {
                        return Err(anyhow!("apply_function: unexpected object: {:?}", obj));
                    }
                }
                None => {
                    return Err(anyhow!("apply_function: unexpected end of list"));
                }
            }
        }

        // function parameters should be next in the list fourth item, index 3
        let parameters = if let Some(obj) = list.step()? {
            obj
        } else {
            return Err(anyhow!(
                "apply_function: fn list terminates before parameters"
            ));
        };

        // function executable should be last in the list fifth item, index 4
        let exe_obj = if let Some(obj) = list.step()? {
            obj
        } else {
            return Err(anyhow!(
                "apply_function: fn list terminates before executeable"
            ));
        };

        // the executable should be a list, of the form
        // (no (id (type x) 'pair))
        // with mixed calls to primatives and to other functions
        let mut exe_list = List::new(&exe_obj);

        // the first element of the list should be the name of a function or
        // a primative
        let inner_name = if let Some(o) = exe_list.step()? {
            if let Object::Symbol(n) = o {
                n
            } else {
                return Err(anyhow!("apply_function: invalid Object: {:?}", o));
            }
        } else {
            return Err(anyhow!("apply_function: exe_list terminates before name"));
        };

        // recursively accumulate the arguments to the inner function or primative
        debug!("apply_function: {}", inner_name);
        let mut inner_args_v: Vec<Object> = Vec::new();
        while let Some(obj) = exe_list.step()? {
            let arg = self.eval(&obj)?;
            inner_args_v.push(arg);
        }
        let inner_args = object::from_vec(inner_args_v)?;
        debug!("apply_function: {} inner_args before replace = {}", inner_name, inner_args);
        let locals = merge_args_with_params(&inner_args, &parameters)?;
        let inner_args = replace_params_with_args(&locals, &parameters)?;
        debug!("apply_function: {} inner_args after replace = {}", inner_name, inner_args);

        if self.function_names.contains(&inner_name) {
            debug!("apply_function: applying inner function {}; args = {}", inner_name, inner_args);
            self.apply_function(&inner_name, &inner_args)
        } else {
            match self.primatives.get(&inner_name) {
                Some(p) => {
                    debug!("apply_function: applying primative: {}, args = {}", inner_name, inner_args);
                    p(&inner_args)
                }
                None => Err(anyhow!("unknown inner_name: {}", inner_name)),
            }
        }
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
            let body = object::from_vec(vec![symbol!("lit"), symbol!("clo"), nil!(), p, e])?;
            Ok((name, body))
        } else {
            Err(anyhow!("invalid def name {:?}", args))
        }
    } else {
        Err(anyhow!("invalid def {:?}", args))
    }
}

fn merge_args_with_params(args: &Object, params: &Object) -> Result<ObjectMap, Error> {
    let mut locals: ObjectMap = HashMap::new();

    let args_v = args.to_vec()?;
    let params_v = params.to_vec()?;

    if args_v.len() > params_v.len() {
        return Err(anyhow!(
            "too many args: {}; for {} params",
            args_v.len(),
            params_v.len()
        ));
    }

    for i in 0..args_v.len() {
        if let Object::Symbol(param_str) = params_v[i].clone() {
            locals.insert(param_str, args_v[i].clone());
        } else {
            return Err(anyhow!("invalid param object: {:?}", params_v[i]));
        }
    }

    // if we have unmatched params, fill with nil
    if args_v.len() < params_v.len() {
        for param in &params_v[args_v.len()..] {
            if let Object::Symbol(param_str) = param {
                locals.insert(param_str.to_string(), nil!());
            } else {
                return Err(anyhow!("invalid param object: {:?}", param));
            }
        }
    }

    Ok(locals)
}

fn replace_params_with_args(locals: &ObjectMap, params: &Object) -> Result<Object, Error> {
    let mut accum: Object = nil!();
    let mut params = params.clone();

    while !params.is_nil() {
        if let Object::Pair(pair) = params {
            let (car, cdr) = *pair.clone();
            let obj = if let Object::Symbol(name) = car.clone() {
                if let Some(arg) = locals.get(&name) {
                    arg
                } else {
                    &car
                }
            } else {
                &car
            };
            accum = object::join(obj.clone(), accum)?;
            params = cdr;
        } else {
            return Err(anyhow!("expected Pair: {:?}", params));
        }
    }

    Ok(accum)
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
        assert_eq!(obj, symbol!("b"));
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
        assert_eq!(obj, pair!(symbol!("x"), nil!()));

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
