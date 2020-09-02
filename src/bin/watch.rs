// TODO support reading from standard input

extern crate extra;
extern crate termion;

use std::env::{self, args};
use std::io::{self, Read, Write};
use std::process::{self, Command, Stdio};
use std::time::Duration;
use std::{cmp, str, thread};

use extra::option::OptionalExt;

use termion::raw::IntoRawMode;
use termion::{async_stdin, clear, cursor, style, terminal_size};

static MAN_PAGE: &str = /* @MANSTART{watch} */ r#"
NAME
    watch - execute a program periodically, showing output fullscreen

SYNOPSIS
    watch [-h | --help] [-n N | --interval N] command

DESCRIPTION
    Runs command repeatedly, displaying its output and errors. This allows you to watch the program
    output change over time. By default, the program is run every 2 seconds. By default, watch will
    run until interrupted.

OPTIONS
    --help, -h
        Print this manual page.

AUTHOR
    This program was written by Jeremy Soller for Redox OS. Bugs, issues, or feature requests
    should be reported in the Gitlab repository, 'redox-os/extrautils'.

COPYRIGHT
    Copyright (c) 2016 Jeremy Soller

    Permission is hereby granted, free of charge, to any person obtaining a copy of this software
    and associated documentation files (the "Software"), to deal in the Software without
    restriction, including without limitation the rights to use, copy, modify, merge, publish,
    distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all copies or
    substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
    BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
    NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
    DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#; /* @MANEND */

fn main() {
    let mut args = args().skip(1);
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let mut command = String::new();
    let mut interval = 2;

    while let Some(x) = args.next() {
        match x.as_str() {
            "--help" | "-h" => {
                // Print help.
                stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
                return;
            }
            "--interval" | "-n" => {
                if let Some(interval_str) = args.next() {
                    if let Ok(interval_num) = interval_str.parse::<u64>() {
                        interval = cmp::max(1, interval_num);
                    } else {
                        eprintln!("watch: interval argument not specified");
                        process::exit(1);
                    }
                } else {
                    eprintln!("watch: interval argument not specified");
                    process::exit(1);
                }
            }
            arg => {
                if !command.is_empty() {
                    command.push(' ');
                }
                command.push_str(arg);
            }
        }
    }

    if command.is_empty() {
        eprintln!("watch: command argument not specified");
        process::exit(1);
    }

    run(command, interval, stdout).try(&mut stderr);
}

fn run<W: IntoRawMode>(command: String, interval: u64, stdout: W) -> std::io::Result<()> {
    let title = format!("Every {}s: {}", interval, command);

    let shell = env::var("SHELL").unwrap_or_else(|_| String::from("sh"));

    let mut stdout = stdout.into_raw_mode()?;

    let (w, h) = terminal_size()?;

    let mut stdin = async_stdin();

    'watching: loop {
        write!(
            stdout,
            "{}{}{}",
            clear::All,
            style::Reset,
            cursor::Goto(1, 1)
        )?;

        let child = Command::new(&shell)
            .arg("-c")
            .arg(&command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let output = String::from_utf8_lossy(&child.stdout);

        for (y, line) in output.lines().enumerate() {
            write!(stdout, "{}", cursor::Goto(1, (y + 1) as u16))?;

            if line.len() > w as usize {
                stdout.write_all(line[..w as usize].as_bytes())?;
            } else {
                stdout.write_all(line.as_bytes())?;
            }

            if (y + 1) as u16 >= h {
                break;
            }
        }

        write!(
            stdout,
            "{}{}{}{}",
            cursor::Goto(1, h),
            style::Invert,
            title,
            style::NoInvert
        )?;

        stdout.flush()?;

        for _second in 0..interval * 10 {
            for b in (&mut stdin).bytes() {
                if b? == b'q' {
                    break 'watching;
                }
            }

            thread::sleep(Duration::new(0, 100000000));
        }
    }

    write!(
        stdout,
        "{}{}{}",
        clear::All,
        style::Reset,
        cursor::Goto(1, 1)
    )?;

    Ok(())
}
