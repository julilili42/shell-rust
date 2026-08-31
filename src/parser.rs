use winnow::{
    ModalResult, Parser,
    ascii::{space0, space1},
    combinator::{alt, opt},
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

pub fn parse_command(input: &mut &str) -> ModalResult<ShellCommand> {
    let _ = opt(space0).parse_next(input)?;

    let cmd_name = take_till(0.., char::is_whitespace).parse_next(input)?;

    let command = match cmd_name {
        "exit" => ShellCommand::Exit,
        "echo" => {
            let (arg, redirect) = parse_argument.parse_next(input)?;
            ShellCommand::Echo(arg, redirect)
        }
        "type" => {
            let (arg, redirect) = parse_argument.parse_next(input)?;
            ShellCommand::Type(arg, redirect)
        }
        "pwd" => {
            let redirect = parse_redirect.parse_next(input)?;
            ShellCommand::Pwd(redirect)
        }
        "cd" => {
            let (arg, redirect) = parse_argument.parse_next(input)?;
            ShellCommand::Cd(arg, redirect)
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
    let op_opt = opt(alt((">>", ">", "1>"))).parse_next(input)?;

    if let Some(op) = op_opt {
        let _ = opt(space0).parse_next(input)?;
        let file = take_till(0.., char::is_whitespace).parse_next(input)?;

        let op_res = match op {
            ">" | "1>" => RedirectOperation::Write,
            ">>" => RedirectOperation::Append,
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

    let arg_str = alt((take_until(0.., "1>"), take_until(0.., ">"), rest)).parse_next(input)?;

    let arg = arg_str.trim_end().to_string();
    let redirect = parse_redirect.parse_next(input)?;

    Ok((arg, redirect))
}
