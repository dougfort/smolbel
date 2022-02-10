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
