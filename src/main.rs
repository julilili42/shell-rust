use std::io::BufRead;
#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    match stdin.lock().lines().next().unwrap() {
        Ok(input) => {
            println!("{}: command not found", input);
        }
        Err(error) => println!("error: {error}"),
    }
    io::stdout().flush().unwrap();
}
