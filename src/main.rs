#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env, fmt::Display, fs, io::BufRead, os::unix::fs::PermissionsExt, path::Path, str::FromStr,
};

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
            s if s.starts_with("type") => Ok(Command::Type(determine_type(s)?)),
            "exit" => Ok(Command::Exit),
            cmd => Err(format!("{}: command not found", cmd).into()),
        }
    }
}

fn determine_type(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let processed = input
        .strip_prefix("type")
        .expect("failed to strip prefix")
        .trim();
    match processed {
        "echo" | "exit" | "type" => Ok(format!("{} is a shell builtin", processed)),
        cmd_name => search_executable(cmd_name),
    }
}

fn search_executable(cmd_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let Some(env_var) = env::var_os("PATH") else {
        return Ok(format!("PATH variable not set"));
    };

    for path in env::split_paths(&env_var) {
        if !path.is_dir() {
            continue;
        }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_name() == cmd_name && can_execute(&entry.path()) {
                return Ok(format!("{cmd_name} is {}", entry.path().display()));
            }
        }
    }

    Ok(format!("{}: not found", cmd_name))
}

fn can_execute(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        let mode = metadata.permissions().mode();
        metadata.is_file() && (mode & 0o111 != 0)
    } else {
        false
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
