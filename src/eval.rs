use crate::functions;
use crate::list::List;
use crate::object;
use crate::object::Object;
use crate::primatives::{load_primatives, PrimFunc};

use anyhow::{anyhow, Error};
use log::{debug, trace};
use std::collections::{HashMap, HashSet};

pub type ObjectMap = HashMap<Object, Object>;
pub fn new_object_map() -> ObjectMap {
    HashMap::new()
}
pub fn dump_object_map(om: &ObjectMap) -> String {
    let mut accum = String::new();

    for (key, val) in  om {
        accum.push_str(&format!(" {} => {}; ", key, val));
    }

    accum
}

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
                (nil!(), nil!()),
                (symbol!("t"), symbol!("t")),
                (symbol!("o"), symbol!("o")),
                (symbol!("apply"), symbol!("apply")),
            ]),
            primatives: load_primatives(),
            function_names: HashSet::new(),
            macro_names: HashSet::new(),
        }
    }

    pub fn eval(&mut self, locals: &ObjectMap, exp: &Object) -> Result<Object, Error> {
        debug!("eval: exp = {}; locals = {}", exp, dump_object_map(locals));
        let output = match exp {
            Object::Symbol(name) => self.get_bound_object(locals, name)?,
            Object::Pair(_) => self.eval_pair(locals, exp)?,
            Object::Char(_c) => {
                return Err(anyhow!("Object::Char not implemented"));
            }
            Object::Stream => {
                return Err(anyhow!("Object::Stream not implemented"));
            }
        };

        trace!("eval: exp = {}; output = {}", exp, output);
        Ok(output)
    }

    fn get_bound_object(&self, locals: &ObjectMap, name: &str) -> Result<Object, Error> {
        let key = symbol!(name);
        match locals.get(&key) {
            Some(obj) => Ok(obj.clone()),
            None => match self.globals.get(&key) {
                Some(obj) => Ok(obj.clone()),
                None => Err(anyhow!("unbound symbol: {:?}", name)),
            },
        }
    }

    fn eval_pair(&mut self, locals: &ObjectMap, pair: &Object) -> Result<Object, Error> {
        let (car, cdr) = pair.extract_pair()?;
        if let Object::Symbol(name) = car.clone() {
            match name.as_ref() {
                "set" => self.set(&cdr),
                "def" => self.def(&cdr),
                "mac" => self.mac(&cdr),
                "if" => self.r#if(locals, &cdr),
                "quote" => quote(&cdr),
                "type" => {
                    let evaluated_list = self.evaluate_list(locals, &cdr)?;
                    self.r#type(&evaluated_list)
                }
                n if self.primatives.contains_key(n) => {
                    debug!("eval: primative: {}", n);
                    let evaluated_list = self.evaluate_list(locals, &cdr)?;
                    self.primatives[n](&evaluated_list)
                }
                n if self.function_names.contains(n) => {
                    let evaluated_list = self.evaluate_list(locals, &cdr)?;
                    self.apply_function(&car, &evaluated_list)
                }
                n if self.macro_names.contains(n) => {
                    let evaluated_list = self.evaluate_list(locals, &cdr)?;
                    self.apply_macro(n, &evaluated_list)
                }
                _ => self.evaluate_list(locals, pair),
            }
        } else {
            self.evaluate_list(locals, pair)
        }
    }

    fn set(&mut self, args: &Object) -> Result<Object, Error> {
        let list = args.to_vec()?;
        for i in 0..list.len() - 1 {
            let key = list[i].clone();
            if key.t() == "symbol" {
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
            let key = list[i].clone();
            if key.t() == "symbol" {
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

    // An if expression with an odd number of arguments
    //  (if a1 a2 a3 a4 ... an)
    // is equivalent to
    //  if a1 then a2 else if a3 then a4 ... else an
    // I.e. the odd numbered arguments are evaluated in order till we either
    // reach the last, or one returns true.  In the former case, its value
    // is returned as the value of the if expression. In the latter, the
    // succeeding argument is evaluated and its value returned.
    //
    // An if expression with an even number of arguments
    //  (if a1 a2 ... an)
    // is equivalent to
    //  (if a1 a2 ... an nil)
    fn r#if(&mut self, locals: &ObjectMap, args: &Object) -> Result<Object, Error> {
        debug!("#if: {}", args);
        let mut list = List::new(args);

        while let Some(odd_item) = list.step()? {
            match list.step()? {
                // even
                Some(even_item) => {
                    trace!("#if: even_item: {}", even_item);
                    let x = self.eval(locals, &odd_item)?;
                    trace!("#if: odd_item: {}", odd_item);
                    if x.is_true() {
                        return self.eval(locals, &even_item);
                    }
                }
                // None here indicates an odd number of arguments
                // so return 'an'
                None => return self.eval(locals, &odd_item),
            }
        }

        // if we make it here, we had an even number of arguments
        // and none of the predicates was satisified
        Ok(nil!())
    }

    fn r#type(&mut self, args: &Object) -> Result<Object, Error> {
        debug!("#type: {}", args);
        // we assume we have a list like (type a), so args is a pair x . nil
        let (car, cdr) = args.extract_pair()?;
        if cdr.is_nil() {
//            let obj = self.eval(locals, &car)?;
            Ok(symbol!(car.t()))
        } else {
            Err(anyhow!("#type: expecting nil cdr: {}", args))
        }
    }

    fn apply_function(&mut self, f_name: &Object, args: &Object) -> Result<Object, Error> {
        let function = if let Some(f) = self.globals.get(f_name) {
            functions::expand_function(f_name, f)?
        } else {
            return Err(anyhow!("unknown function {}", f_name));
        };

        let locals = merge_args_with_params(args, &function.parameters)?;
        debug!(
            "apply_function: f_name= {}, args= {}, locals = {}",
            f_name, args, dump_object_map(&locals),
        );

        // the function expression should be a list, of the form
        // (no (id (type x) 'pair))
        // with mixed calls to primatives and to other functions
        // the first element of the list should be the name of a function or
        // a primative
        self.eval(&locals, &function.body)
    }

    fn apply_macro(&mut self, _name: &str, _args: &Object) -> Result<Object, Error> {
        Err(anyhow!("apply_macro not implemented"))
    }

    fn evaluate_list(&mut self, locals: &ObjectMap, o: &Object) -> Result<Object, Error> {
        debug!("evaluate_list: locals = {}; o = {}", dump_object_map(locals), o);
        let mut accum: Object = nil!();
        let mut list = List::new(o);

        while let Some(obj) = list.step()? {
            let eval_obj = self.eval(locals, &obj)?;
            accum = object::join(eval_obj, accum)?;
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
fn quote(pair: &Object) -> Result<Object, Error> {
    let (car, cdr) = pair.extract_pair()?;
    if cdr.is_nil() {
        Ok(car)
    } else {
        Err(anyhow!(
            "quote expecting single element list; found {:?}",
            cdr
        ))
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
    debug!(
        "merge_args_with_params: args = {}, params = {}",
        args, params
    );
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
        let key = params_v[i].clone();
        if key.t() == "symbol" {
            debug!(
                "merge_args_with_params: args_v[{}]={}, params_v[{}]={}",
                i, args_v[i], i, params_v[i]
            );
            locals.insert(key, args_v[i].clone());
        } else {
            return Err(anyhow!("invalid param object: {:?}", params_v[i]));
        }
    }

    // if we have unmatched params, fill with nil
    if args_v.len() < params_v.len() {
        for param in &params_v[args_v.len()..] {
            if param.t() == "symbol" {
                locals.insert(param.clone(), nil!());
            } else {
                return Err(anyhow!("invalid param object: {:?}", param));
            }
        }
    }

    trace!("merge_args_with_params: locals {}", dump_object_map(&locals));
    Ok(locals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn can_get_object() -> Result<(), Error> {
        let mut bel = Bel::new();
        let exp = parser::parse("t")?;
        let obj = bel.eval(&new_object_map(), &exp)?;
        assert_eq!(exp, obj);
        Ok(())
    }

    #[test]
    fn can_set_object() -> Result<(), Error> {
        let mut bel = Bel::new();
        let exp = parser::parse("(set a b)")?;
        let obj = bel.eval(&new_object_map(), &exp)?;
        assert!(obj.is_nil());

        let exp = parser::parse("a")?;
        let obj = bel.eval(&new_object_map(), &exp)?;
        assert_eq!(obj, symbol!("b"));
        Ok(())
    }

    #[test]
    fn can_set_multiple() -> Result<(), Error> {
        let mut bel = Bel::new();

        let parse_obj = parser::parse("(set a b c d e f)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil());

        for (key, val) in &[
            ("a", "b".to_string()),
            ("c", "d".to_string()),
            ("e", "f".to_string()),
        ] {
            let parse_obj = parser::parse(key)?;
            let obj = bel.eval(&new_object_map(), &parse_obj)?;
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

        let parse_obj = parser::parse("(set a b c d e)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil());

        for (key, val) in &[
            ("a", "b".to_string()),
            ("c", "d".to_string()),
            ("e", "nil".to_string()),
        ] {
            let parse_obj = parser::parse(key)?;
            let obj = bel.eval(&new_object_map(), &parse_obj)?;
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

        let parse_obj = parser::parse("(set a b)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil());

        let parse_obj = parser::parse("(quote a)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        if let Object::Symbol(s) = obj {
            assert_eq!(s, "a");
        } else {
            panic!("unexpected object {:?}", obj);
        }

        let parse_obj = parser::parse("(quote ( x ))")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert_eq!(obj, pair!(symbol!("x"), nil!()));

        Ok(())
    }

    #[test]
    fn can_def_a_function() -> Result<(), Error> {
        let mut bel = Bel::new();

        let parse_obj = parser::parse(
            r#"(def xnox (x)
                (id x nil))
          "#,
        )?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil());

        let parse_obj = parser::parse("(xnox nil)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_true());

        let parse_obj = parser::parse("(xnox `a)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil(), "{:?}", obj);

        Ok(())
    }

    #[test]
    fn can_evaluate_if() -> Result<(), Error> {
        let mut bel = Bel::new();

        let parse_obj = parser::parse("(if t 'a 'b)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_symbol("a"));

        let parse_obj = parser::parse("(if nil 'a 'b)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_symbol("b"));

        let parse_obj = parser::parse("(if nil 'a)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil());

        let parse_obj = parser::parse("(if nil 'a nil 'b 'c)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_symbol("c"));

        let parse_obj = parser::parse("(if (id nil nil) 'a 'b)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_symbol("a"));

        Ok(())
    }

    #[test]
    fn can_recurse_function() -> Result<(), Error> {
        let mut bel = Bel::new();

        let parse_obj = parser::parse(
            r#"(def rrr (xs)
                       (if nil      t
                           (cdr xs))
          "#,
        )?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil());

        let parse_obj = parser::parse("(rrr nil)")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil());

        let parse_obj = parser::parse("(rrr ('a))")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil());

        let parse_obj = parser::parse("(rrr ('a 'b 'c))")?;
        let obj = bel.eval(&new_object_map(), &parse_obj)?;
        assert!(obj.is_nil());

        Ok(())
    }

    #[test]
    fn can_merge_args_with_params() -> Result<(), Error> {
        let args_v = vec![symbol!("a")];
        let args = object::from_vec(args_v)?;
        let params_v = vec![symbol!("x")];
        let params = object::from_vec(params_v)?;
        let locals = merge_args_with_params(&args, &params)?;
        assert!(locals.contains_key(&symbol!("x")));
        assert!(locals.get(&symbol!("x")) == Some(&symbol!("a")));

        let l1_v = vec![symbol!("a")];
        let l1 = object::from_vec(l1_v)?;
        let args_v = vec![symbol!("no"), l1];
        let args = object::from_vec(args_v)?;
        let params_v = vec![symbol!("f"), symbol!("xs")];
        let params = object::from_vec(params_v)?;
        let locals = merge_args_with_params(&args, &params)?;
        assert!(locals.contains_key(&symbol!("f")));
        assert!(locals.get(&symbol!("f")) == Some(&symbol!("no")));
        assert!(locals.contains_key(&symbol!("xs")));
        assert!(locals.get(&symbol!("xs")) == Some(&pair!(symbol!("a"), nil!())));
        
        Ok(())
    }
}
