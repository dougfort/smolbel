use crate::object::Object;
use anyhow::{anyhow, Error};

#[derive(Debug)]
pub struct List {
    obj: Object,
}

impl List {
    pub fn new(obj: &Object) -> Self {
        List { obj: obj.clone() }
    }

    // we don't implement iter becasue we want to return an error if neccessary
    pub fn step(&mut self) -> Result<Option<Object>, Error> {
        if self.obj.is_nil() {
            Ok(None)
        } else if let Object::Pair(pair) = &self.obj {
            let (car, cdr) = *pair.clone();
            self.obj = cdr;
            Ok(Some(car))
        } else {
            Err(anyhow!("list: invalid Object: {:?}", self.obj))
        }
    }
}

/// convert a list to a string,
pub fn format_list(obj: &Object) -> Result<String, Error> {
    let mut accum = String::new();
    accum_list(&mut accum, obj)?;
    Ok(accum)
}

fn accum_list(accum: &mut String, pair: &Object) -> Result<(), Error> {
    let mut list = List::new(pair);
    accum.push('(');
    while let Some(obj) = list.step()? {
        accum.push(' ');
        if let Object::Pair(_) = obj {
            accum_list(accum, &obj)?;
        } else {
            accum.push_str(&format!("{}", obj));
        }
    }
    accum.push(' ');
    accum.push(')');

    Ok(())
}
