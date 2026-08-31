use winnow::{
    ModalResult, Parser,
    ascii::{space0, space1},
    combinator::{alt, delimited, opt},
    error::{ContextError, ErrMode},
    token::{rest, take_till, take_until},
};

use crate::ShellCommand;

#[derive(Debug)]
pub enum RedirectOperation {
    Write,
    Append,
}
#[derive(Debug)]
pub struct Redirect {
    pub op: RedirectOperation,
    pub file: String,
}

fn parse_word(input: &mut &str) -> ModalResult<String> {
    alt((
        // '...'
        delimited('\'', take_until(0.., '\''), '\'').map(|s: &str| s.to_string()),
        // "..."
        delimited('"', take_until(0.., '"'), '"').map(|s: &str| s.to_string()),
        // unquoted
        take_till(1.., char::is_whitespace).map(|s: &str| s.to_string()),
    ))
    .parse_next(input)
}
pub fn parse_command(input: &mut &str) -> ModalResult<ShellCommand> {
    let _ = opt(space0).parse_next(input)?;

    let cmd_name = parse_word.parse_next(input)?;

    let command = match cmd_name.as_str() {
        "exit" => ShellCommand::Exit,
        "echo" => {
            let (arg, redirect) = parse_argument.parse_next(input)?;
            let text = shell_words::split(&arg)
                .map(|parts| parts.join(" "))
                .unwrap_or(arg);
            ShellCommand::Echo(text, redirect)
        }
        "type" => {
            let (arg, redirect) = parse_argument.parse_next(input)?;
            let target = shell_words::split(&arg)
                .ok()
                .and_then(|parts| parts.into_iter().next())
                .unwrap_or(arg);
            ShellCommand::Type(target, redirect)
        }
        "pwd" => {
            let redirect = parse_redirect.parse_next(input)?;
            ShellCommand::Pwd(redirect)
        }
        "cd" => {
            let (arg, redirect) = parse_argument.parse_next(input)?;
            let path = shell_words::split(&arg)
                .ok()
                .and_then(|parts| parts.into_iter().next())
                .unwrap_or(arg);
            ShellCommand::Cd(path, redirect)
        }
        other => {
            let (arg, redirect) = parse_argument.parse_next(input)?;
            let args = shell_words::split(arg.as_str())
                .map_err(|_| ErrMode::Backtrack(ContextError::new()))?;
            ShellCommand::Unknown(other.to_string(), args, redirect)
        }
    };

    Ok(command)
}

fn parse_redirect(input: &mut &str) -> ModalResult<Option<Redirect>> {
    let _ = opt(space0).parse_next(input)?;
    let op_opt = opt(alt(("1>>", ">>", "1>", ">"))).parse_next(input)?;

    if let Some(op) = op_opt {
        let _ = opt(space0).parse_next(input)?;
        let file = parse_word.parse_next(input)?;

        let op_res = match op {
            ">" | "1>" => RedirectOperation::Write,
            ">>" | "1>>" => RedirectOperation::Append,
            _ => unreachable!(),
        };

        return Ok(Some(Redirect {
            op: op_res,
            file: file.to_string(),
        }));
    };

    return Ok(None);
}

fn parse_argument(input: &mut &str) -> ModalResult<(String, Option<Redirect>)> {
    let space = opt(space1).parse_next(input)?;

    if space.is_none() {
        return Ok((String::new(), None));
    }

    let arg_str = alt((
        take_until(0.., "1>>"),
        take_until(0.., ">>"),
        take_until(0.., "1>"),
        take_until(0.., ">"),
        rest,
    ))
    .parse_next(input)?;

    let arg = arg_str.trim_end().to_string();
    let redirect = parse_redirect.parse_next(input)?;

    Ok((arg, redirect))
}
