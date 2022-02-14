#[cfg(test)]
mod tests {
    use anyhow::Error;

    use smolbel::{load_source, new_object_map, parse, Bel};

    const SOURCE_PATH: &str = "bel_source/bel.bel";
    const LIMIT: usize = 3;

    #[test]
    fn can_load() -> Result<(), Error> {
        env_logger::init();

        let mut bel = Bel::new();
        load_source(&mut bel, SOURCE_PATH, Some(LIMIT))?;

        // expression #1
        let exp = parse("(no `a)")?;
        let obj = bel.eval(&new_object_map(), &exp)?;
        assert!(obj.is_nil(), "obj.is_nil(): {:?}", obj);

        // expression #1
        let exp = parse("(no nil)")?;
        let obj = bel.eval(&new_object_map(), &exp)?;
        assert!(obj.is_true());

        // expression #2
        let exp = parse("(atom `a)")?;
        let obj = bel.eval(&new_object_map(), &exp)?;
        assert!(obj.is_true(), "obj.is_true(): {:?}", obj);

        let exp = parse("(atom `(a))")?;
        let obj = bel.eval(&new_object_map(), &exp)?;
        assert!(
            obj.is_nil(),
            "obj.is_nil
        : {:?}",
            obj
        );

        // expression #3
        // expression #3
        let exp = parse("(all (no (nil nil nil)))")?;
        let obj = bel.eval(&new_object_map(), &exp)?;
        assert!(obj.is_symbol("t"));

        Ok(())
    }
}
