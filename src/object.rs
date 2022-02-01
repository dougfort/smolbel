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

    pub fn nil() -> Self {
        Object::Symbol("nil".to_string()) 
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
