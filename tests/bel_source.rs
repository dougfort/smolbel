#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Error;
    use smolbel::{load_source, object, parse, Bel, Object};

    const SOURCE_PATH: &str = "bel_source/bel.bel";

    #[test]
    fn can_load() -> Result<(), Error> {
        let mut bel = Bel::new();
        load_source(&mut bel, SOURCE_PATH)?;

        let exp = parse("(no `a)")?;
        let locals: HashMap<String, Object> = HashMap::new();
        let obj = bel.eval(&locals, &exp)?;
        assert_eq!(obj, object::nil());

        let exp = parse("(no nil)")?;
        let locals: HashMap<String, Object> = HashMap::new();
        let obj = bel.eval(&locals, &exp)?;
        assert_eq!(obj, object::symbol("t"));

        Ok(())
    }
}
