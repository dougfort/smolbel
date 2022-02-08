use crate::parse;
use crate::Bel;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use log::{trace, debug, warn};

pub fn load_source(bel: &mut Bel, filepath: &str) -> Result<()> {
    debug!(":loading {}", filepath);
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);

    let mut expr_count: usize = 0;
    let mut accum = String::new();
    'line_loop: for line in reader.lines() {
        let line = line?;
        trace!("load_source: line = {:?}", line);
        // https://sep.yimg.com/ty/cdn/paulgraham/bellanguage.txt has some weird
        // unicode byte order mark
        if !line.starts_with('\u{feff}') && !line.starts_with(';') {
            if line.is_empty() && !accum.is_empty() {
                debug!("expr = {:?}", accum);
                let parsed_expr = parse(&accum)?;
                if parsed_expr.is_nil() {
                    debug!("skipping empty expression");
                    continue 'line_loop;
                }
                bel.eval(&parsed_expr).context(format!("\n\n{}\n", accum))?;
                expr_count += 1;
                if expr_count == 1 {
                    warn!("load_source: breaking after {} expression", expr_count);
                    break;
                }
                accum.clear();
            }

            accum.push_str(&format!("{}\n", &line));
        }
    }

    Ok(())
}
