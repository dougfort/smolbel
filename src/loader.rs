use crate::parse;
use crate::{new_object_map, Bel};
use anyhow::{Context, Result};
use log::{debug, trace, warn};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn load_source(bel: &mut Bel, filepath: &str, limit: Option<usize>) -> Result<()> {
    debug!(":loading {} limit = {:?}", filepath, limit);
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
                    warn!("skipping empty expression");
                    continue 'line_loop;
                }
                bel.eval(&new_object_map(), &parsed_expr)
                    .context(format!("\n\n{}\n", accum))?;
                expr_count += 1;
                if let Some(limit) = limit {
                    if expr_count == limit {
                        warn!("load_source: breaking after {} expression", expr_count);
                        break;
                    }
                }
                accum.clear();
            }

            accum.push_str(&format!("{}\n", &line));
        }
    }

    Ok(())
}
