use std::io::BufRead;
#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let input = io::stdin().lock().lines().next().expect("new line");

        match input {
            Ok(input) => {
                if input == "exit".to_string() {
                    break;
                }
                println!("{}: command not found", input);
            }
            Err(error) => println!("error: {error}"),
        }
        io::stdout().flush().unwrap();
    }
}
