#[allow(unused_imports)]
use std::io::{self, Write};
use std::{fmt::Display, io::BufRead, str::FromStr};

enum Command {
    Exit,
    Echo(String),
    Type(String),
}
impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Echo(_) => write!(f, "echo"),
            Command::Exit => write!(f, "exit"),
            Command::Type(_) => write!(f, "type"),
        }
    }
}

impl FromStr for Command {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.starts_with("echo") => Ok(Command::Echo(print_echo(s))),
            s if s.starts_with("type") => Ok(Command::Type(determine_type(s))),
            "exit" => Ok(Command::Exit),
            cmd => Err(format!("{}: command not found", cmd).into()),
        }
    }
}

fn determine_type(input: &str) -> String {
    let processed = input
        .strip_prefix("type")
        .expect("failed to strip prefix")
        .trim();
    match processed {
        "echo" | "exit" | "type" => format!("{} is a shell builtin", processed),
        _ => format!("{}: not found", processed),
    }
}

fn print_echo(input: &str) -> String {
    input
        .strip_prefix("echo")
        .expect("failed to strip prefix")
        .trim()
        .to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let input = io::stdin().lock().lines().next().expect("new line");
        let processed = input.map(|s| Command::from_str(&s));

        match processed {
            Ok(Ok(cmd)) => match cmd {
                Command::Echo(echo) => println!("{}", echo),
                Command::Exit => break,
                Command::Type(t) => println!("{}", t),
            },
            Ok(Err(e)) => {
                println!("{e}");
                continue;
            }
            Err(error) => println!("error: {error}"),
        }

        io::stdout().flush().unwrap();
    }
    Ok(())
}
