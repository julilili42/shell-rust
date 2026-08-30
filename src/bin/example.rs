use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::space0;
use winnow::ascii::space1;
use winnow::combinator::{alt, opt};
use winnow::token::rest;
use winnow::token::take_till;
use winnow::token::take_until;

#[derive(Debug)]
enum ShellCommand {
    Exit,
    Echo(String, Option<Redirect>),
    Type(String, Option<Redirect>),
    Pwd(Option<Redirect>),
    Cd(String, Option<Redirect>),
    Unknown(String),
}

#[derive(Debug)]
enum RedirectOperation {
    Write,
    Append,
}
#[derive(Debug)]
struct Redirect {
    op: RedirectOperation,
    file: String,
}

fn parse_command(input: &mut &str) -> ModalResult<ShellCommand> {
    let _ = opt(space0).parse_next(input)?;

    let c = alt(("exit", "echo", "type", "pwd", "cd")).parse_next(input)?;

    let command = match c {
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
        _ => unreachable!(),
    };

    Ok(command)
}

fn parse_redirect(input: &mut &str) -> ModalResult<Option<Redirect>> {
    let _ = opt(space0).parse_next(input)?;
    let op_opt = opt(alt((">>", ">"))).parse_next(input)?;

    if let Some(op) = op_opt {
        let _ = opt(space0).parse_next(input)?;
        let file = take_till(0.., char::is_whitespace).parse_next(input)?;

        let op_res = match op {
            ">" => RedirectOperation::Write,
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
    let _ = space1.parse_next(input)?;

    let arg_str = alt((take_until(0.., ">"), rest)).parse_next(input)?;

    let arg = arg_str.trim_end().to_string();
    let redirect = parse_redirect.parse_next(input)?;

    Ok((arg, redirect))
}

fn main() {
    let mut input = "echo Hallo > output.txt";
    let output = parse_command.parse_next(&mut input);
    println!("{:?}", output);
}
