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
                } else if input.starts_with("echo") {
                    println!(
                        "{}",
                        input
                            .strip_prefix("echo")
                            .expect("failed to strip prefix")
                            .trim()
                    );
                } else {
                    println!("{}: command not found", input);
                }
            }
            Err(error) => println!("error: {error}"),
        }
        io::stdout().flush().unwrap();
    }
}
