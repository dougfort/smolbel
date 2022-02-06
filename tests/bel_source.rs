#[cfg(test)]
mod tests {
    use anyhow::Error;
    use smolbel::{load_source, object, parse, Bel};

    const SOURCE_PATH: &str = "bel_source/bel.bel";

    #[test]
    fn can_load() -> Result<(), Error> {
        let mut bel = Bel::new();
        load_source(&mut bel, SOURCE_PATH)?;

        let exp = parse("(no `a)")?;
        let obj = bel.eval(&exp)?;
        assert_eq!(obj, object::nil());

        let exp = parse("(no nil)")?;
        let obj = bel.eval(&exp)?;
        assert_eq!(obj, object::symbol("t"));

        Ok(())
    }
}
