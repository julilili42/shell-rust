# Rusty Shell

A small interactive Unix shell written in Rust as a learning project.

## Features

- Built-ins: `cd`, `echo`, `exit`, `pwd`, and `type`
- Runs executables from `PATH` or an explicit path
- Single- and double-quoted arguments
- Standard output and error redirection with `>`, `>>`, `1>`, `1>>`, `2>`, and `2>>`

## Run

Rust 1.96 or newer is required.

```sh
cargo run
```

```text
$ echo "hello world"
hello world
$ type echo
echo is a shell builtin
$ pwd
/your/current/directory
```

This is not a fully POSIX-compliant shell. Pipes, environment expansion, and job control are not implemented.
