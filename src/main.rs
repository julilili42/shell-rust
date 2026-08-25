#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    ffi::OsStr,
    fmt::Display,
    fs,
    io::BufRead,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

enum ShellCommand {
    Exit,
    Echo(String),
    Type(String),
    Pwd,
    Unknown(String),
}
impl Display for ShellCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellCommand::Echo(_) => write!(f, "echo"),
            ShellCommand::Exit => write!(f, "exit"),
            ShellCommand::Type(_) => write!(f, "type"),
            ShellCommand::Pwd => write!(f, "pwd"),
            ShellCommand::Unknown(cmd) => write!(f, "{cmd}"),
        }
    }
}

impl FromStr for ShellCommand {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.starts_with("echo") => Ok(ShellCommand::Echo(print_echo(s))),
            s if s.starts_with("type") => Ok(ShellCommand::Type(determine_type(s)?)),
            "exit" => Ok(ShellCommand::Exit),
            "pwd" => Ok(ShellCommand::Pwd),
            s => Ok(ShellCommand::Unknown(s.to_string())),
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
        cmd_name => search_executable(cmd_name, OsStr::new("PATH")),
    }
}
fn is_env_executable(
    cmd_name: &str,
    env_var: &OsStr,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let var_os = env::var_os(env_var).ok_or_else(|| format!("env variable not set"))?;

    for path in env::split_paths(&var_os) {
        if !path.is_dir() {
            continue;
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_name() == cmd_name && can_execute(&entry.path()) {
                return Ok(Some(entry.path()));
            }
        }
    }
    Ok(None)
}
fn search_executable(
    cmd_name: &str,
    env_var: &OsStr,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(path) = is_env_executable(&cmd_name, &env_var)? {
        return Ok(format!("{cmd_name} is {}", path.display()));
    } else {
        return Ok(format!("{}: not found", cmd_name));
    }
}

fn can_execute(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        let mode = metadata.permissions().mode();
        metadata.is_file() && (mode & 0o111 != 0)
    } else {
        false
    }
}

fn start_executable(cmd: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut args_list: Vec<&str> = cmd.split(" ").map(|a| a.trim()).collect();
    let exec_name = args_list.get(0).expect("exec name missing");
    if let Some(_) = is_env_executable(&exec_name, OsStr::new("PATH"))? {
        let mut child = Command::new(exec_name).args(&mut args_list[1..]).spawn()?;
        child.wait()?;
    } else {
        println!("{}: command not found", cmd)
    }
    Ok(())
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
        let processed = input.map(|s| ShellCommand::from_str(&s))?;

        match processed {
            Ok(cmd) => match cmd {
                ShellCommand::Echo(echo) => println!("{}", echo),
                ShellCommand::Exit => break,
                ShellCommand::Pwd => {
                    println!("{}", env::current_dir()?.display())
                }
                ShellCommand::Type(t) => println!("{}", t),
                ShellCommand::Unknown(cmd) => start_executable(cmd)?,
            },
            Err(e) => {
                println!("{e}");
                continue;
            }
        }

        io::stdout().flush().unwrap();
    }
    Ok(())
}
