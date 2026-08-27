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
    Cd(String),
    Unknown(String),
}
impl Display for ShellCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellCommand::Echo(_) => write!(f, "echo"),
            ShellCommand::Exit => write!(f, "exit"),
            ShellCommand::Type(_) => write!(f, "type"),
            ShellCommand::Pwd => write!(f, "pwd"),
            ShellCommand::Cd(_) => write!(f, "cd"),
            ShellCommand::Unknown(cmd) => write!(f, "{cmd}"),
        }
    }
}

impl FromStr for ShellCommand {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = shell_words::split(s)?;
        let args_list: Vec<&str> = split.iter().map(|s| s.as_str()).collect();

        match args_list.as_slice() {
            ["echo", args @ ..] => Ok(ShellCommand::Echo(args.join(" "))),
            ["type", args @ ..] => Ok(ShellCommand::Type(determine_type(args.join(" "))?)),
            ["cd", args @ ..] => Ok(ShellCommand::Cd(get_path(args.join(" "))?)),
            ["exit"] => Ok(ShellCommand::Exit),
            ["pwd"] => Ok(ShellCommand::Pwd),
            cmd => Ok(ShellCommand::Unknown(shell_words::join(cmd))),
        }
    }
}

fn get_path(path: String) -> Result<String, Box<dyn std::error::Error>> {
    if path == "~" {
        let home = env::var("HOME")?;
        return Ok(home);
    }

    let clean_path = fs::canonicalize(&path)?;

    Ok(clean_path
        .to_str()
        .ok_or("pathbuf to string conversion failed")?
        .to_string())
}

fn determine_type(input: String) -> Result<String, Box<dyn std::error::Error>> {
    match input.as_str() {
        "echo" | "exit" | "type" | "pwd" | "cd" => Ok(format!("{} is a shell builtin", input)),
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
    let mut args = shell_words::split(cmd.as_str())?;
    let exec_name = args.get(0).expect("exec name missing");
    if let Some(_) = is_env_executable(&exec_name, OsStr::new("PATH"))? {
        let mut child = Command::new(exec_name).args(&mut args[1..]).spawn()?;
        child.wait()?;
    } else {
        println!("{}: command not found", cmd)
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let input = io::stdin().lock().lines().next().ok_or("failed to parse")?;
        let processed = input.map(|s| ShellCommand::from_str(&s))?;
        match processed {
            Ok(cmd) => match cmd {
                ShellCommand::Echo(echo) => println!("{}", echo),
                ShellCommand::Exit => break,
                ShellCommand::Pwd => {
                    println!("{}", env::current_dir()?.display())
                }
                ShellCommand::Cd(s) => {
                    let path = PathBuf::from(s);
                    match path.try_exists() {
                        Ok(true) => env::set_current_dir(path)?,
                        _ => println!("cd: {}: No such file or directory", path.display()),
                    }
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
