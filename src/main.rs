use std::collections::HashMap;

use anyhow::{anyhow, Error};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use smolbel::{load_source, parse, Bel, Object};

fn main() -> Result<(), Error> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    };

    let mut bel = Bel::new();

    'repl_loop: loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                if line.starts_with(':') {
                    process_repl_command(&mut bel, &line);
                    continue 'repl_loop;
                }

                let locals: HashMap<String, Object> = HashMap::new();
                match parse(&line) {
                    Ok(exp) => {
                        println!("parsed exp = {:?}", exp);
                        match bel.eval(&locals, &exp) {
                            Ok(obj) => {
                                println!("eval output = {:?}", obj);
                            }
                            Err(err) => {
                                eprintln!("error: {:?}", err);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("error: {:?}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                return Err(anyhow!("Error from readline: {:?}", err));
            }
        }
    }
    rl.save_history("history.txt").unwrap();

    Ok(())
}

fn process_repl_command(bel: &mut Bel, line: &str) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts[0] {
        ":global" | ":globals" => {
            println!("global");
            for (key, value) in &bel.globals {
                println!("({}, {:?}", key, value);
            }
        }
        ":load" => {
            if parts.len() != 2 {
                println!("load: <filepah>");
                return;
            }
            match load_source(bel, parts[1]) {
                Ok(()) => {}
                Err(err) => {
                    println!("error: during :load; {:?}", err);
                }
            }
        }
        _ => {
            println!("error: unkbnown REPL command {}", line);
        }
    }
}
