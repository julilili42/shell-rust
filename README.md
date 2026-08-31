# Shell Rust

Rusty Shell is a small interactive Unix shell built in Rust for the [CodeCrafters "Build Your Own Shell" challenge](https://codecrafters.io/challenges/shell). It supports built-in commands, `PATH` lookup, quoted arguments, and output redirection.

## Roadmap

- [x] Built-in commands
  - [x] `cd`
  - [x] `echo`
  - [x] `exit`
  - [x] `pwd`
  - [x] `type`
- [x] Run executables from `PATH` or an explicit path
- [x] Parse single- and double-quoted arguments
- [x] Redirect standard output and standard error
  - [x] Overwrite with `>`, `1>`, and `2>`
  - [x] Append with `>>`, `1>>`, and `2>>`
- [ ] Pipes
- [ ] Environment variable expansion
- [ ] Job control

## Run

Install [Rust](https://www.rust-lang.org/tools/install). Rust 1.96 or newer is required.

```bash
cargo run
```

Then enter commands at the prompt:

```text
$ echo "hello world"
hello world
$ type echo
echo is a shell builtin
$ echo "saved output" > message.txt
$ cat message.txt
saved output
$ exit
```

## Platforms

- macOS and Linux — supported
- Windows — not supported; executable detection uses Unix permission bits

## Architecture

```text
stdin
  │
  ▼
command parser
  │
  ▼
ShellCommand
  ├── built-in ──▶ execute in the shell
  └── external ──▶ find in PATH ──▶ start process
                                      │
                                      ▼
                              terminal or file
```

| Module          | Responsibility                                      |
| --------------- | --------------------------------------------------- |
| `src/parser.rs` | Parses commands, arguments, quotes, and redirections |
| `src/main.rs`   | Runs the prompt, built-ins, and external executables |

## References

- [CodeCrafters: Build Your Own Shell](https://codecrafters.io/challenges/shell)
- [`winnow` parser documentation](https://docs.rs/winnow/)
- [`shell-words` documentation](https://docs.rs/shell-words/)

Rusty Shell is a learning project and is not fully POSIX-compliant.
