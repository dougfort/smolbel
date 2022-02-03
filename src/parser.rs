use crate::object;
use crate::object::Object;
use anyhow::{anyhow, Error};

struct ParseState {
    remainder: String,
    obj: Object,
}

pub fn parse(text: &str) -> Result<Object, Error> {
    let mut obj_accum: Vec<Object> = Vec::new();

    let mut data = text.to_string();
    while !data.is_empty() {
        let state = dispatch_char(text)?;
        data = state.remainder;
        if !state.obj.is_nil() {
            obj_accum.push(state.obj);
        }
    }

    match obj_accum.len() {
        0 => Ok(object::nil()),
        1 => Ok(obj_accum[0].clone()),
        _ => Err(anyhow!("multiple objects: {:?}", obj_accum)),
    }
}

fn dispatch_char(text: &str) -> Result<ParseState, Error> {
    let state = consume_whitespace(text)?;
    if state.remainder.is_empty() {
        Ok(ParseState {
            remainder: text.to_string(),
            obj: object::nil(),
        })
    } else if state.remainder.starts_with('(') {
        consume_parens(&state.remainder)
    } else if state.remainder.starts_with(')') {
        Ok(ParseState {
            remainder: text[1..].to_string(),
            obj: state.obj,
        })
    } else if state.remainder.starts_with('\\') {
        consume_char(&state.remainder)
    } else {
        consume_symbol(&state.remainder)
    }
}

fn consume_whitespace(text: &str) -> Result<ParseState, Error> {
    for (i, c) in text.chars().enumerate() {
        if !c.is_whitespace() {
            return Ok(ParseState {
                remainder: text[i..].to_string(),
                obj: object::nil(),
            });
        }
    }

    Ok(ParseState {
        remainder: "".to_string(),
        obj: object::nil(),
    })
}

fn consume_parens(text: &str) -> Result<ParseState, Error> {
    if text.is_empty() {
        return Err(anyhow!("consume_parens called with empty text"));
    }

    if !text.starts_with('(') {
        return Err(anyhow!(
            "consume_parens text does not start with '(' '{}'",
            text
        ));
    }
    let mut data = text[1..].to_string();

    let mut vec_accum: Vec<Object> = Vec::new();

    'dispatch_loop: while !data.is_empty() {
        let state = dispatch_char(&data)?;
        if !state.obj.is_nil() {
            vec_accum.push(state.obj);
        }
        if state.remainder.starts_with(')') {
            data = state.remainder[1..].to_string();
            break 'dispatch_loop;
        }
        data = state.remainder;
    }

    // the list we accumulated while parsing is backwards
    vec_accum.reverse();

    let mut obj_accum: Object = object::nil();
    for obj in vec_accum {
        obj_accum = object::join(obj, obj_accum)?;
    }

    Ok(ParseState {
        remainder: data,
        obj: obj_accum,
    })
}

fn consume_char(text: &str) -> Result<ParseState, Error> {
    let mut accum = String::new();
    let mut i: usize = 0;

    'char_loop: for c in text.chars() {
        if is_boundaray_char(c) {
            break 'char_loop;
        }
        i += 1;
        accum.push(c);
    }

    let obj = if accum.is_empty() {
        object::nil()
    } else {
        object::symbol(&accum)
    };

    Ok(ParseState {
        remainder: text[i..].to_string(),
        obj,
    })
}

fn consume_symbol(text: &str) -> Result<ParseState, Error> {
    let mut accum = String::new();
    let mut i: usize = 0;

    'char_loop: for c in text.chars() {
        if c.is_whitespace() || c == '(' || c == ')' {
            break 'char_loop;
        }
        i += 1;
        accum.push(c);
    }

    let obj = if accum.is_empty() {
        object::nil()
    } else {
        object::symbol(&accum)
    };

    Ok(ParseState {
        remainder: text[i..].to_string(),
        obj,
    })
}

fn is_boundaray_char(c: char) -> bool {
    c.is_whitespace() || c == '(' || c == ')' || c == '\\' || c == '`'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_consume_whitespace() -> Result<(), Error> {
        let parse_state = consume_symbol("")?;
        assert!(parse_state.remainder.is_empty());
        assert!(parse_state.obj.is_nil());

        let parse_state = consume_symbol("a")?;
        assert!(parse_state.remainder.is_empty());
        assert!(parse_state.obj.is_symbol("a"));

        let parse_state = consume_whitespace("(")?;
        assert_eq!(parse_state.remainder, "(");
        assert!(parse_state.obj.is_nil());

        Ok(())
    }

    #[test]
    fn can_consume_symbol() -> Result<(), Error> {
        let parse_state = consume_whitespace("")?;
        assert!(parse_state.remainder.is_empty());
        assert!(parse_state.obj.is_nil());

        let parse_state = consume_whitespace("a")?;
        assert_eq!(parse_state.remainder, "a");
        assert!(parse_state.obj.is_nil());

        let parse_state = consume_whitespace("    (")?;
        assert_eq!(parse_state.remainder, "(");
        assert!(parse_state.obj.is_nil());

        Ok(())
    }

    #[test]
    fn can_consume_parens() -> Result<(), Error> {
        let parse_state = consume_parens("()")?;
        assert!(parse_state.remainder.is_empty());
        assert!(parse_state.obj.is_nil());

        let parse_state = consume_parens("( a )")?;
        assert!(
            parse_state.remainder.is_empty(),
            "remainder.is_empty() {:?}",
            parse_state.remainder
        );
        assert_eq!(parse_state.obj.to_vec()?, vec![object::symbol("a")]);

        let parse_state = consume_parens("( a b )")?;
        assert!(
            parse_state.remainder.is_empty(),
            "remainder.is_empty() {:?}",
            parse_state.remainder
        );
        assert_eq!(
            parse_state.obj.to_vec()?,
            vec![object::symbol("a"), object::symbol("b")]
        );

        let parse_state = consume_parens("( a b (c d))")?;
        assert!(
            parse_state.remainder.is_empty(),
            "remainder.is_empty() {:?}",
            parse_state.remainder
        );
        assert_eq!(
            parse_state.obj.to_vec()?,
            vec![
                object::symbol("a"),
                object::symbol("b"),
                object::pair(
                    object::symbol("c"),
                    object::pair(object::symbol("d"), object::nil())
                )

                ]
        );

        Ok(())
    }
}
