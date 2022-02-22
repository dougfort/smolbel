use crate::list::List;
use crate::object::Object;
use anyhow::{anyhow, Error, Result};

pub struct Function {
    pub name: String,
    pub parameters: Object,
    pub body: Object,
}

pub fn parse_function(f_name: &Object, f_obj: &Object) -> Result<Function, Error> {
    let mut list = List::new(f_obj);

    // we expect the function to contain 5 items
    // starting with the symbols lit clo nil
    for &name in &["lit", "clo", "nil"] {
        match list.step()? {
            Some(obj) => {
                if let Object::Symbol(symbol_name) = obj.clone() {
                    if symbol_name != name {
                        return Err(anyhow!(
                            "parse_function: unexpected symbol: {:?}; expected {}",
                            obj,
                            name
                        ));
                    }
                } else {
                    return Err(anyhow!("parse_function: unexpected object: {:?}", obj));
                }
            }
            None => {
                return Err(anyhow!("parse_function: unexpected end of list"));
            }
        }
    }

    // function parameters should be next in the list fourth item, index 3
    let parameters = if let Some(obj) = list.step()? {
        obj
    } else {
        return Err(anyhow!(
            "parse_function: fn list terminates before parameters"
        ));
    };

    // function body should be last in the list fifth item, index 4
    let body = if let Some(obj) = list.step()? {
        obj
    } else {
        return Err(anyhow!("parse_function: fn list terminates before body"));
    };

    Ok(Function {
        name: f_name.to_string(),
        parameters,
        body,
    })
}
