use anyhow::{anyhow, Error};
use log::info;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use smolbel::{load_source, new_object_map, parse, Bel, List, Object};

fn main() -> Result<(), Error> {
    env_logger::init();
    info!("program starts");

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

                match parse(&line) {
                    Ok(exp) => {
                        println!("parsed exp = {}", exp);
                        match bel.eval(&new_object_map(), &exp) {
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
            println!("globals");
            for key in bel.globals.keys() {
                println!("{}", key);
            }
        }
        ":primative" | ":primatives" => {
            println!("primatives");
            for key in bel.primatives.keys() {
                println!("{}", key);
            }
        }
        ":function" | ":functions" => {
            println!("functions");
            for key in &bel.function_names {
                println!("{}", key);
            }
        }
        ":load" => {
            // TODO: parse parts[3] for limit
            if parts.len() != 2 {
                println!("load: <filepah>");
                return;
            }
            match load_source(bel, parts[1], Some(3)) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("error: during :load; {:?}", err);
                }
            }
        }
        ":get" => {
            if parts.len() != 2 {
                println!("get: <key>");
                return;
            }
            match bel.globals.get(parts[1]) {
                Some(obj) => {
                    println!("{}", obj);
                }
                None => {
                    eprintln!("error: unknown key: {}", line);
                }
            }
        }
        ":fn" => {
            if parts.len() != 2 {
                println!("fn: <name>");
                return;
            }
            let name = parts[1];
            if !bel.function_names.contains(name) {
                eprintln!("{} is not a function", name);
                return;
            }
            match bel.globals.get(name) {
                Some(obj) => match dump_list(obj, 0) {
                    Ok(()) => {}
                    Err(err) => {
                        eprintln!("dump_list failed: {:?}", err);
                    }
                },
                None => {
                    eprintln!("error: unknown key: {}", line);
                }
            }
        }
        _ => {
            eprintln!("error: unknown REPL command {}", line);
        }
    }
}

fn dump_list(obj: &Object, level: usize) -> Result<(), Error> {
    let mut list = List::new(obj);

    while let Some(obj) = list.step()? {
        if obj.t() == "pair" {
            dump_list(&obj, level + 1)?;
        } else {
            println!("{} {}", str::repeat(" ", level * 4), obj);
        }
    }

    Ok(())
}
