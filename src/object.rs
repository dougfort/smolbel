use anyhow::{anyhow, Error};
use std::fmt;

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

/// nil object (aka 'false')
macro_rules! nil {
    () => {
        Object::Symbol("nil".to_string())
    };
}

/// 'true' object (true is not a good name for a macro)
macro_rules! t {
    () => {
        Object::Symbol("t".to_string())
    };
}

/// general symbol
macro_rules! symbol {
    ($n:expr) => {
        Object::Symbol($n.to_string())
    };
}

/// pair, probably part of a list
macro_rules! pair {
    ($a:expr, $b:expr) => {
        Object::Pair(Box::new(($a, $b)))
    };
}

/// character
macro_rules! char {
    ($n:expr) => {
        Object::Char($n.to_string())
    };
}

/// stream
macro_rules! stream {
    () => {
        Object::Stream
    };
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Object::Symbol(name) => {
                write!(f, "{}", name)?;
            }
            Object::Pair(pair) => {
                let p = *pair.clone();
                let (h, t) = p;
                write!(f, "({} . {})", h, t)?;
            }
            Object::Char(c) => {
                write!(f, "c({})", c)?;
            }
            Object::Stream => {
                write!(f, "stream")?;
            }
        }
        Ok(())
    }
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

    // anything other thn 'nil is true
    pub fn is_true(&self) -> bool {
        !self.is_nil()
    }

    pub fn is_pair(&self, o1: Object, o2: Object) -> bool {
        if let Object::Pair(pair) = self {
            let (car, cdr) = *pair.clone();
            o1 == car && o2 == cdr
        } else {
            false
        }
    }

    pub fn extract_pair(&self) -> Result<(Object, Object), Error> {
        if let Object::Pair(pair) = self {
            Ok(*pair.clone())
        } else {
            Err(anyhow!("expecting pair found: {:?}", self))
        }
    }

    // convert a series of pairs (proper list) into a vector
    // note that this does not handle an embedded list
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

// join puts a an object at the head of the list
// see also cons, which takes multiple objects
pub fn join(obj: Object, list: Object) -> Result<Object, Error> {
    if list.is_nil() {
        Ok(Object::Pair(Box::new((obj, nil!()))))
    } else if let Object::Pair(_) = list {
        Ok(Object::Pair(Box::new((obj, list))))
    } else {
        Err(anyhow!("invalid list: {:?}", list))
    }
}

pub fn from_vec(v: Vec<Object>) -> Result<Object, Error> {
    let mut mv = v;
    mv.reverse();

    let mut obj_accum: Object = nil!();
    for obj in mv {
        obj_accum = join(obj, obj_accum)?;
    }

    Ok(obj_accum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_join_list() -> Result<(), Error> {
        // append nil to empty list. this may be an error
        // (join nil, nil)
        let list = join(nil!(), nil!())?;
        assert!(list.is_pair(nil!(), nil!()));
        let v = list.to_vec()?;
        assert_eq!(v, vec![nil!()]);

        // append a single object to an empty list
        // (join a nil)
        let list = join(symbol!("a"), nil!())?;
        assert!(list.is_pair(symbol!("a"), nil!()));
        let v = list.to_vec()?;
        assert_eq!(v, vec![symbol!("a")]);

        // append a single object to an existing list
        // (join a nil)
        let list = join(symbol!("b"), list)?;
        assert!(list.is_pair(symbol!("b"), pair!(symbol!("a"), nil!())));
        let v = list.to_vec()?;
        assert_eq!(v, vec![symbol!("b"), symbol!("a")]);

        Ok(())
    }
    #[test]
    fn can_build_list_from_vec() -> Result<(), Error> {
        let o = from_vec(vec![])?;
        assert!(o.is_nil());
        assert_eq!(o.to_vec()?, vec![]);

        let o = from_vec(vec![nil!()])?;
        assert_eq!(o, pair!(nil!(), nil!()));
        assert_eq!(o.to_vec()?, vec![nil!()]);

        let o = from_vec(vec![nil!(), nil!()])?;
        assert_eq!(o, pair!(nil!(), pair!(nil!(), nil!())));
        assert_eq!(o.to_vec()?, vec![nil!(), nil!()]);

        Ok(())
    }
}
