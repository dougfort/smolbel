#[cfg(test)]
mod tests {
    use anyhow::Error;

    use smolbel::eval;
    use smolbel::loader;
    use smolbel::parser;

    const SOURCE_PATH: &str = "bel_source/bel.bel";
    const LIMIT: usize = 3;

    #[test]
    fn can_load() -> Result<(), Error> {
        env_logger::init();

        let mut bel = eval::Bel::new();
        loader::load_source(&mut bel, SOURCE_PATH, Some(LIMIT))?;

        // expression #1
        let exp = parser::parse("(no `a)")?;
        let obj = bel.eval(&eval::new_object_map(), &exp)?;
        assert!(obj.is_nil(), "obj.is_nil(): {:?}", obj);

        // expression #1
        let exp = parser::parse("(no nil)")?;
        let obj = bel.eval(&eval::new_object_map(), &exp)?;
        assert!(obj.is_true());

        // expression #2
        let exp = parser::parse("(atom `a)")?;
        let obj = bel.eval(&eval::new_object_map(), &exp)?;
        assert!(obj.is_true(), "obj.is_true(): {:?}", obj);

        let exp = parser::parse("(atom `(a))")?;
        let obj = bel.eval(&eval::new_object_map(), &exp)?;
        assert!(obj.is_nil(), "obj.is_nil: {:?}", obj);

        // expression #3
        // expression #3
        let exp = parser::parse("(all (no (t t)))")?;
        let obj = bel.eval(&eval::new_object_map(), &exp)?;
        assert!(obj.is_symbol("t"));

        Ok(())
    }
}
