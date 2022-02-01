use anyhow::{anyhow, Error};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use smolbel::Bel;

fn main() -> Result<(), Error> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    };

    let mut bel = Bel::new(); 

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match bel.eval(&line) {
                    Ok(obj) =>  {
                        println!("{:?}", obj);
                    },
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
