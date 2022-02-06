use crate::parse;
use crate::Bel;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn load_source(bel: &mut Bel, filepath: &str) -> Result<()> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);

    let mut expr_count: usize = 0;
    let mut accum = String::new();
    for line in reader.lines() {
        let line = line?;
        println!("load_source: line = {:?}", line);
        // https://sep.yimg.com/ty/cdn/paulgraham/bellanguage.txt has some weird
        // unicode byte order mark
        if !line.starts_with('\u{feff}') && !line.starts_with(';') {
            if line.is_empty() && !accum.is_empty() {
                let parsed_expr = parse(&accum)?;
                bel.eval(&parsed_expr).context(format!("\n\n{}\n", accum))?;
                expr_count += 1;
                if expr_count == 1 {
                    println!("load_source: breaking after {} expression", expr_count);
                    break;
                }
                accum.clear();
            }

            accum.push_str(&format!("{}\n", &line));
        }
    }

    Ok(())
}
