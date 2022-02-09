# Simple Rust Macros

I don't often think of macros as a resource in Rust. But I've been playing with a Rust substrate for Paul Graham's [Bel](http://www.paulgraham.com/bel.html) dialect of Lisp.

```rust
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
```

There's a lot of code like this: 
```
Object::Pair(Object::Symbol("a".to_string()), Object::Symbol("nil".to_string()))
```

In [Rust for Rustaceans](https://rust-for-rustaceans.com/), Jon Gjengset says:
> Declarative macros are primarily useful when you find yourself writing the same code over and over, and you'd like to, well, not do that.

So I added some simple macros. I got a lot of help from  [Rust by Example](https://doc.rust-lang.org/rust-by-example/macros.html)

```rust
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
```

Now I write:
```rust
pair!(symbol!("a"), nil!())
```

Much better, and not nearly as hard as I expected.

You can see the WIP code [here](https://github.com/dougfort/smolbel)