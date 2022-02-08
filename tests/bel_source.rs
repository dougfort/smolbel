#[cfg(test)]
mod tests {
    use anyhow::Error;

    use smolbel::{load_source, parse, Bel};

    const SOURCE_PATH: &str = "bel_source/bel.bel";

    #[test]
    fn can_load() -> Result<(), Error> {
        env_logger::init();
        
            let mut bel = Bel::new();
        load_source(&mut bel, SOURCE_PATH)?;

        let exp = parse("(no `a)")?;
        let obj = bel.eval(&exp)?;
        assert!(obj.is_nil(), "obj.is_nil(): {:?}", obj);

        let exp = parse("(no nil)")?;
        let obj = bel.eval(&exp)?;
        assert!(obj.is_true());

        Ok(())
    }
}
