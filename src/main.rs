#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    ffi::OsStr,
    fs,
    io::BufRead,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use winnow::Parser;

use crate::parser::{Redirect, RedirectOperation, parse_command};

mod parser;

#[derive(Debug)]
enum ShellCommand {
    Exit,
    Echo(String, Option<Redirect>),
    Type(String, Option<Redirect>),
    Pwd(Option<Redirect>),
    Cd(String, Option<Redirect>),
    Unknown(String, Vec<String>, Option<Redirect>),
}

impl FromStr for ShellCommand {
    type Err = Box<dyn std::error::Error>;
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let cmd = parse_command
            .parse_next(&mut s)
            .map_err(|e| format!("parsing error: {}", e))?;

        match cmd {
            ShellCommand::Echo(arg, redirect) => Ok(ShellCommand::Echo(arg, redirect)),
            ShellCommand::Type(arg, redirect) => {
                Ok(ShellCommand::Type(determine_type(arg)?, redirect))
            }
            ShellCommand::Cd(arg, redirect) => Ok(ShellCommand::Cd(get_path(arg)?, redirect)),
            ShellCommand::Unknown(cmd, args, redirect) => {
                Ok(ShellCommand::Unknown(cmd, args, redirect))
            }
            ShellCommand::Pwd(redirect) => Ok(ShellCommand::Pwd(redirect)),
            cmd => Ok(cmd),
        }
    }
}

fn get_path(path: String) -> Result<String, Box<dyn std::error::Error>> {
    if path == "~" {
        let home = env::var("HOME")?;
        return Ok(home);
    }

    let Ok(clean_path) = fs::canonicalize(&path) else {
        return Err(format!("cd: {}: No such file or directory", path).into());
    };

    Ok(clean_path
        .to_str()
        .ok_or("pathbuf to string conversion failed")?
        .to_string())
}

fn determine_type(cmd: String) -> Result<String, Box<dyn std::error::Error>> {
    match cmd.as_str() {
        "echo" | "exit" | "type" | "pwd" | "cd" => Ok(format!("{} is a shell builtin", cmd)),
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

fn start_executable(
    cmd: String,
    mut args: Vec<String>,
    redirect: Option<Redirect>,
) -> Result<(), Box<dyn std::error::Error>> {
    let executable_path = if Path::new(&cmd).is_file() {
        Some(PathBuf::from(&cmd))
    } else {
        is_env_executable(&cmd, OsStr::new("PATH"))?
    };

    match executable_path {
        Some(path) => {
            let output = Command::new(&path).args(&mut args).output()?;
            let output_str = String::from_utf8_lossy(&output.stdout);
            let error_str = String::from_utf8_lossy(&output.stderr);

            if !error_str.is_empty() {
                eprint!("{}", error_str);
                io::stderr().flush()?;
            }

            if let Some(ref r) = redirect {
                write_to_redirect(&r, &output_str)?;
            } else {
                print!("{}", output_str);
                io::stdout().flush()?;
            }
        }
        None => println!("{}: command not found", &cmd),
    }
    Ok(())
}

fn output_result(
    content: &str,
    redirect: Option<Redirect>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(r) = redirect {
        let formatted_content = format!("{}\n", content);
        write_to_redirect(&r, &formatted_content)?;
    } else {
        println!("{}", content);
        io::stdout().flush()?;
    }
    Ok(())
}

fn write_to_redirect(r: &Redirect, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    match r.op {
        RedirectOperation::Write => fs::write(&r.file, content)?,
        RedirectOperation::Append => {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&r.file)?;
            file.write_all(content.as_bytes())?;
            file.write_all(b"\n")?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let input = io::stdin().lock().lines().next().ok_or("failed to parse")?;
        let processed = input.map(|s| ShellCommand::from_str(&s))??;
        match processed {
            ShellCommand::Echo(echo, redirect) => output_result(&format!("{}", echo), redirect)?,
            ShellCommand::Pwd(redirect) => {
                output_result(&format!("{}", env::current_dir()?.display()), redirect)?
            }
            ShellCommand::Cd(s, redirect) => {
                let path = PathBuf::from(s);
                env::set_current_dir(path)?;
                if let Some(r) = redirect {
                    write_to_redirect(&r, "")?;
                }
            }
            ShellCommand::Type(t, redirect) => output_result(&format!("{}", t), redirect)?,
            ShellCommand::Unknown(cmd, args, redirect) => {
                start_executable(cmd, args, redirect)?;
            }
            ShellCommand::Exit => break,
        };
    }

    io::stdout().flush().unwrap();

    Ok(())
}
