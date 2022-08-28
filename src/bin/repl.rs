use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("jr {}", option_env!("CARGO_PKG_VERSION").unwrap());

    loop {
        // repl
        stdout.write(b"   ")?; //prompt
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;

        match buffer.trim() {
            "exit" => break,
            _sentence => {
                match jr::scan(&buffer) {
                    Ok(tokens) => {
                        println!("tokens: {:?}", tokens);
                        println!("{:?}", jr::eval(tokens));
                    }
                    Err(e) => println!("error: {:?}", e),
                }
                buffer = String::from("");
            }
        }
    }

    Ok(())
}
