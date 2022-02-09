#[cfg(test)]
mod tests {
    use anyhow::Error;

    use smolbel::{load_source, parse, Bel};

    const SOURCE_PATH: &str = "bel_source/bel.bel";
    const LIMIT: usize = 2;

    #[test]
    fn can_load() -> Result<(), Error> {
        env_logger::init();

        let mut bel = Bel::new();
        load_source(&mut bel, SOURCE_PATH, Some(LIMIT))?;

        // expression #1
        let exp = parse("(no `a)")?;
        let obj = bel.eval(&exp)?;
        assert!(obj.is_nil(), "obj.is_nil(): {:?}", obj);

        // expression #1
        let exp = parse("(no nil)")?;
        let obj = bel.eval(&exp)?;
        assert!(obj.is_true());

        // expression #2
        let exp = parse("(atom `a)")?;
        let obj = bel.eval(&exp)?;
        assert!(obj.is_true(), "obj.is_true(): {:?}", obj);

        let exp = parse("(atom `(a))")?;
        let obj = bel.eval(&exp)?;
        assert!(
            obj.is_nil(),
            "obj.is_nil
        : {:?}",
            obj
        );

        Ok(())
    }
}
