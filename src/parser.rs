use crate::object;
use crate::object::Object;
use anyhow::{anyhow, Error};

struct ParseState {
    remainder: String,
    obj: Option<Object>,
}

pub fn parse(text: &str) -> Result<Object, Error> {
    let mut obj_accum: Vec<Object> = Vec::new();

    let mut data = text.to_string();
    while !data.is_empty() {
        let state = dispatch_char(&data)?;
        data = state.remainder;
        if let Some(obj) = state.obj {
            obj_accum.push(obj);
        }
    }

    match obj_accum.len() {
        0 => Ok(nil!()),
        1 => Ok(obj_accum[0].clone()),
        _ => Err(anyhow!("multiple objects: {:?}", obj_accum)),
    }
}

fn dispatch_char(text: &str) -> Result<ParseState, Error> {
    let state = consume_whitespace(text)?;
    if state.remainder.is_empty() {
        Ok(ParseState {
            remainder: "".to_string(),
            obj: None,
        })
    } else if state.remainder.starts_with('(') {
        consume_parens(&state.remainder)
    } else if state.remainder.starts_with(')') {
        Ok(ParseState {
            remainder: text[1..].to_string(),
            obj: state.obj,
        })
    // the  spec https://sep.yimg.com/ty/cdn/paulgraham/bellanguage.txt?t=1595850613&
    // defines slightly different usages for backtick and single quote
    // but I haven't figured that out
    } else if state.remainder.starts_with('`') || state.remainder.starts_with('\''){
        consume_quote(&state.remainder)
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
                obj: None,
            });
        }
    }

    Ok(ParseState {
        remainder: "".to_string(),
        obj: None,
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
        if let Some(obj) = state.obj {
            vec_accum.push(obj);
        }
        if state.remainder.starts_with(')') {
            data = state.remainder[1..].to_string();
            break 'dispatch_loop;
        }
        data = state.remainder;
    }

    let obj_accum = object::from_vec(vec_accum)?;

    Ok(ParseState {
        remainder: data,
        obj: Some(obj_accum),
    })
}

fn consume_quote(text: &str) -> Result<ParseState, Error> {
    if text.is_empty() {
        return Err(anyhow!("consume_quote called with empty text"));
    }

    if !(text.starts_with('`') || text.starts_with('\'') ) {
        return Err(anyhow!(
            "consume_quote text does not start with '`' or '\'' '{}'",
            text
        ));
    }
    let state = dispatch_char(&text[1..].to_string())?;
    match state.obj {
        Some(obj) => {
            let mut obj_accum: Object = nil!();
            obj_accum = object::join(obj, obj_accum)?;
            obj_accum = object::join(symbol!("quote"), obj_accum)?;

            Ok(ParseState {
                remainder: state.remainder,
                obj: Some(obj_accum),
            })
        }
        None => Err(anyhow!("consume_quote: quoted nil object")),
    }
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
        None
    } else {
        Some(symbol!(&accum))
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
        None
    } else {
        Some(symbol!(&accum))
    };

    Ok(ParseState {
        remainder: text[i..].to_string(),
        obj,
    })
}

fn is_boundaray_char(c: char) -> bool {
    c.is_whitespace() || c == '(' || c == ')' || c == '\\' || c == '`' || c == '\''
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_consume_symbol() -> Result<(), Error> {
        let parse_state = consume_symbol("")?;
        assert!(parse_state.remainder.is_empty());
        assert!(parse_state.obj.is_none());

        let parse_state = consume_symbol("a")?;
        assert!(parse_state.remainder.is_empty());
        assert!(parse_state.obj.unwrap().is_symbol("a"));

        let parse_state = consume_whitespace("(")?;
        assert_eq!(parse_state.remainder, "(");
        assert!(parse_state.obj.is_none());

        Ok(())
    }

    #[test]
    fn can_consume_whitespacel() -> Result<(), Error> {
        let parse_state = consume_whitespace("")?;
        assert!(parse_state.remainder.is_empty());
        assert!(parse_state.obj.is_none());

        let parse_state = consume_whitespace("a")?;
        assert_eq!(parse_state.remainder, "a");
        assert!(parse_state.obj.is_none());

        let parse_state = consume_whitespace("    (")?;
        assert_eq!(parse_state.remainder, "(");
        assert!(parse_state.obj.is_none());

        Ok(())
    }

    #[test]
    fn can_consume_parens() -> Result<(), Error> {
        let parse_state = consume_parens("()")?;
        assert!(parse_state.remainder.is_empty());
        assert!(parse_state.obj.unwrap().is_nil());

        let parse_state = consume_parens("( a )")?;
        assert!(
            parse_state.remainder.is_empty(),
            "remainder.is_empty() {:?}",
            parse_state.remainder
        );
        assert_eq!(parse_state.obj.unwrap().to_vec()?, vec![symbol!("a")]);

        let parse_state = consume_parens("( a b )")?;
        assert!(
            parse_state.remainder.is_empty(),
            "remainder.is_empty() {:?}",
            parse_state.remainder
        );
        assert_eq!(
            parse_state.obj.unwrap().to_vec()?,
            vec![symbol!("a"), symbol!("b")]
        );

        let parse_state = consume_parens("( a b (c d))")?;
        assert!(
            parse_state.remainder.is_empty(),
            "remainder.is_empty() {:?}",
            parse_state.remainder
        );
        assert_eq!(
            parse_state.obj.unwrap().to_vec()?,
            vec![
                symbol!("a"),
                symbol!("b"),
                pair!(symbol!("c"), pair!(symbol!("d"), nil!()))
            ]
        );

        let parse_state = consume_parens("( a nil )")?;
        assert!(
            parse_state.remainder.is_empty(),
            "remainder.is_empty() {:?}",
            parse_state.remainder
        );
        assert_eq!(
            parse_state.obj.unwrap(),
            pair!(symbol!("a"), pair!(nil!(), nil!()))
        );

        Ok(())
    }

    #[test]
    fn can_consume_quote() -> Result<(), Error> {
        let parse_state = consume_quote("`a")?;
        assert!(parse_state.remainder.is_empty());
        assert_eq!(
            parse_state.obj.unwrap().to_vec()?,
            vec![symbol!("quote"), symbol!("a")]
        );

        let parse_state = consume_quote("`(a)")?;
        assert!(parse_state.remainder.is_empty());
        assert_eq!(
            parse_state.obj.unwrap().to_vec()?,
            vec![symbol!("quote"), pair!(symbol!("a"), nil!())]
        );

        Ok(())
    }

    #[test]
    fn can_parse_list_of_nil() -> Result<(), Error> {
        let obj = parse("()")?;
        assert!(obj.is_nil(), "obj.is_nil() {:?}", obj);

        let obj = parse("(nil)")?;
        assert_eq!(obj, pair!(nil!(), nil!()));

        Ok(())
    }
}
