use anyhow::{anyhow, Error, Result};
use log::info;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use smolbel::eval;
use smolbel::list;
use smolbel::loader;
use smolbel::object;
use smolbel::parser;
use smolbel::functions;

struct State {
    text: String,
    bel: eval::Bel,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    info!("program starts");

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    };

    let mut state = State {
        text: String::new(),
        bel: eval::Bel::new(),
    };

    'repl_loop: loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                if line.starts_with(':') {
                    match process_repl_command(&mut state, &line) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("error: {:?}", err);
                        }
                    }
                    continue 'repl_loop;
                }

                match parser::parse(&line) {
                    Ok(exp) => {
                        println!("parsed exp = {}", exp);
                        match state.bel.eval(&eval::new_object_map(), &exp) {
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

fn process_repl_command(state: &mut State, line: &str) -> Result<(), Error> {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    match parts[0] {
        ":global" | ":globals" => {
            println!("globals");
            for key in state.bel.globals.keys() {
                println!("{}", key);
            }
        }
        ":primative" | ":primatives" => {
            println!("primatives");
            for key in state.bel.primatives.keys() {
                println!("{}", key);
            }
        }
        ":function" | ":functions" => {
            println!("functions");
            for key in &state.bel.function_names {
                println!("{}", key);
            }
        }
        ":load" => {
            // TODO: parse parts[3] for limit
            if parts.len() != 2 {
                return Err(anyhow!("invalid command").context(":load <filepah>"));
            };
            loader::load_source(&mut state.bel, parts[1], Some(3))?;
        }
        ":get" => {
            if parts.len() != 2 {
                return Err(anyhow!("invalid command").context(":get <key>"));
            }
            match state
                .bel
                .globals
                .get(&object::Object::Symbol(parts[1].to_string()))
            {
                Some(obj) => {
                    println!("{}", obj);
                }
                None => {
                    return Err(anyhow!("unknown key: {}", line));
                }
            }
        }
        ":fn" => {
            if parts.len() != 2 {
                return Err(anyhow!("invalid command").context(":fn <name>"));
            }
            let name = object::Object::Symbol(parts[1].to_string());
            if !state.bel.function_names.contains(parts[1]) {
                return Err(anyhow!("{} is not a function", name));
            }
            let obj = state
                .bel
                .globals
                .get(&name)
                .ok_or_else(|| anyhow!("unknown name {}", name))?;
            dump_list(obj, 0)?;
        }
        ":parse" => {
            if parts.len() != 2 {
                return Err(anyhow!("invalid command").context(":parse <code>"));
            }
            state.text = parts[1].to_string();
            let obj = parser::parse(&state.text)?;
            println!("{}", obj);
        }
        ":eval" => {
            if parts.len() != 2 {
                return Err(anyhow!("invalid command").context(":eval <code>"));
            }
            state.text = parts[1].to_string();
            let obj = parser::parse(&state.text)?;
            let (exp_name, _args) = obj.extract_pair()?;
            let function = if let Some(f) = state.bel.globals.get(&exp_name) {
                functions::expand_function(&exp_name, f)?
            } else {
                return Err(anyhow!("unknown function {}", exp_name));
            };
            println!("parameters = {}", list::format_list(&function.parameters)?);
            println!("body = {}", list::format_list(&function.body)?);
            parse_body(&function.body)?;
        }
        _ => {
            return Err(anyhow!("unknown REPL command {}", line));
        }
    }

    Ok(())
}

fn parse_body(obj: &object::Object) -> Result<(), Error> {
    let (car, cdr) = obj.extract_pair()?;
    println!("body car = {:?}", car);
    println!("body cdr = {:?}", cdr);
    Ok(())
}

fn dump_list(obj: &object::Object, level: usize) -> Result<(), Error> {
    let mut list = list::List::new(obj);

    while let Some(obj) = list.step()? {
        if obj.t() == "pair" {
            dump_list(&obj, level + 1)?;
        } else {
            println!("{} {}", str::repeat(" ", level * 4), obj);
        }
    }

    Ok(())
}
