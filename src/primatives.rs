use std::collections::HashMap;

use anyhow::{Result, Error};
use crate::object::Object;
use crate::object;

pub type PrimFunc = fn(&Object) -> Result<Object, Error>;

pub fn load_primatives() -> HashMap<String, PrimFunc> {
    HashMap::from([
        ("id".to_string(), id as PrimFunc),
    ])
}

fn id(params: &Object) -> Result<Object, Error> {
    // id is true if
    // * there are two arguments
    // * they are both symbols
    // they have the same name
    let mut result = object::nil();
    let p = params.to_vec()?;
    if p.len() == 2 {
        if let Object::Symbol(lhs) = &p[0] {
            if let Object::Symbol(rhs) = &p[1] {
                if lhs == rhs {
                    result = Object::Symbol("t".to_string());
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn can_check_id() -> Result<(), Error> {
        let params = parser::parse("('a 'a)")?;
        let ans = id(&params)?;
        assert!(ans.is_true());

        let params = parser::parse("('a 'b)")?;
        let ans = id(&params)?;
        assert!(!ans.is_true());

        Ok(())
    }

}
