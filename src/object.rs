use anyhow::{anyhow, Error};

/// Bel has four fundamental data types:
/// symbols, pairs, characters, and streams.
/// Instances of the four fundamental types are called objects
/// https://sep.yimg.com/ty/cdn/paulgraham/bellanguage.txt
#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Symbol(String),
    Pair(Box<(Object, Object)>),
    Char(String),
    Stream,
}

impl Object {
    pub fn is_symbol(&self, name: &str) -> bool {
        if let Object::Symbol(n) = self {
            n == name
        } else {
            false
        }
    }

    pub fn is_nil(&self) -> bool {
        if let Object::Symbol(name) = self {
            name == "nil"
        } else {
            false
        }
    }

    pub fn is_pair(&self, o1: Object, o2: Object) -> bool {
        if let Object::Pair(pair) = self {
            let (car, cdr) = *pair.clone();
            o1 == car && o2 == cdr
        } else {
            false
        }
    }

    // convert a series of pairs (proper list) into a vector
    pub fn to_vec(&self) -> Result<Vec<Object>, Error> {
        let mut accum: Vec<Object> = Vec::new();
        let mut list = self.clone();

        while !list.is_nil() {
            if let Object::Pair(pair) = list {
                let (car, cdr) = *pair.clone();
                accum.push(car);
                list = cdr;
            } else {
                return Err(anyhow!("expected Pair: {:?}", self));
            }
        }

        Ok(accum)
    }

    pub fn t(&self) -> String {
        match &self {
            Object::Symbol(_) => "symbol".to_string(),
            Object::Pair(_) => "pair".to_string(),
            Object::Char(_) => "char".to_string(),
            Object::Stream => "stream".to_string(),
        }
    }
}

pub fn symbol(name: &str) -> Object {
    Object::Symbol(name.to_string())
}

pub fn nil() -> Object {
    symbol("nil")
}

pub fn pair(a: Object, b: Object) -> Object {
    Object::Pair(Box::new((a, b)))
}

// join puta an object at the head of the list
// see also cons, which takes multiple objects
pub fn join(obj: Object, list: Object) -> Result<Object, Error> {
    if list.is_nil() {
        Ok(Object::Pair(Box::new((obj, nil()))))
    } else if let Object::Pair(_) = list {
        Ok(Object::Pair(Box::new((obj, list))))
    } else {
        Err(anyhow!("invalid list: {:?}", list))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_join_list() -> Result<(), Error> {
        // append nil to empty list. this may be an error
        // (join nil, nil)
        let list = join(nil(), nil())?;
        assert!(list.is_pair(nil(), nil()));
        let v = list.to_vec()?;
        assert_eq!(v, vec![nil()]);

        // append a single object to an empty list
        // (join a nil)
        let list = join(symbol("a"), nil())?;
        assert!(list.is_pair(symbol("a"), nil()));
        let v = list.to_vec()?;
        assert_eq!(v, vec![symbol("a")]);

        // append a single object to an existing list
        // (join a nil)
        let list = join(symbol("b"), list)?;
        assert!(list.is_pair(symbol("b"), pair(symbol("a"), nil())));
        let v = list.to_vec()?;
        assert_eq!(v, vec![symbol("b"), symbol("a")]);

        Ok(())
    }
}
