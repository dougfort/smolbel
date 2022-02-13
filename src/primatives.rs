use std::collections::HashMap;

use crate::object::Object;
use anyhow::{anyhow, Error, Result};
use log::debug;

pub type PrimFunc = fn(&Object) -> Result<Object, Error>;

pub fn load_primatives() -> HashMap<String, PrimFunc> {
    HashMap::from([
        ("id".to_string(), id as PrimFunc),
        ("car".to_string(), car as PrimFunc),
        ("cdr".to_string(), cdr as PrimFunc),
    ])
}

fn id(params: &Object) -> Result<Object, Error> {
    // id is true if
    // * there are two arguments
    // * they are both symbols
    // they have the same name
    debug!("id: params = {}", params);

    let p_v = params.to_vec()?;
    if p_v.len() != 2 {
        return Err(anyhow!("id: invalid number of params: {}", params));
    }

    let mut result = nil!();
    if let Object::Symbol(lhs) = &p_v[0] {
        if let Object::Symbol(rhs) = &p_v[1] {
            if lhs == rhs {
                result = t!();
            }
        }
    };

    Ok(result)
}

fn car(params: &Object) -> Result<Object, Error> {
    debug!("car: params = {}", params);
    let (car, _) = params.extract_pair()?;
    Ok(car)
}

fn cdr(params: &Object) -> Result<Object, Error> {
    debug!("cdr: params = {}", params);
    let (_, cdr) = params.extract_pair()?;
    Ok(cdr)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn can_check_id() -> Result<(), Error> {
        let params = parser::parse("(a a)")?;
        let ans = id(&params)?;
        assert!(ans.is_true());

        let params = parser::parse("(a b)")?;
        let ans = id(&params)?;
        assert!(!ans.is_true());

        Ok(())
    }
}
