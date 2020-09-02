// TODO support reading from standard input

extern crate extra;
extern crate termion;

use std::env::args;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str::Chars;

use extra::option::OptionalExt;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style, terminal_size};

static MAN_PAGE: &str = /* @MANSTART{mdless} */ r#"
NAME
    mdless- view a markdown file.

SYNOPSIS
    mdless [-h | --help] [input]

DESCRIPTION
    This utility views md files. If no input file is specified as an argument, standard input is
    used.

OPTIONS
    --help, -h
        Print this manual page.

AUTHOR
    This program was written by MovingtoMars and Jeremy Soller for Redox OS. Bugs, issues, or feature requests should
    be reported in the Gitlab repository, 'redox-os/extrautils'.

COPYRIGHT
    Copyright (c) 2016 MovingtoMars, Jeremy Soller

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

#[cfg(target_os = "redox")]
fn terminal_path() -> String {
    use std::env;
    env::var("TTY").unwrap()
}

#[cfg(not(target_os = "redox"))]
fn terminal_path() -> String {
    "/dev/tty".to_string()
}

fn main() {
    let mut args = args().skip(1).peekable();
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stderr = io::stderr();

    if let Some(x) = args.peek() {
        if x == "--help" || x == "-h" {
            // Print help.
            print!("{}", MAN_PAGE);
            return;
        }
    } else {
        let mut terminal = File::open(terminal_path()).try(&mut stderr);
        run(PathBuf::from("-"), &mut stdin, &mut terminal, &mut stdout).try(&mut stderr);
    };

    for filename in args {
        let filepath = PathBuf::from(filename.as_str());
        let file = File::open(&filepath);
        match file {
            Ok(mut open_file) => {
                if let Err(err) = run(filepath, &mut open_file, &mut stdin, &mut io::stdout()) {
                    eprintln!("{}: {}", &filename, err);
                }
            }
            Err(err) => {
                eprintln!("{}: {}", &filename, err);
            }
        }
    }
}

//TODO: String in Text
enum Block {
    Text(char),
    Bold(Vec<Block>),
    Italic(Vec<Block>),
    Code(Vec<Block>),
    Link(Vec<Block>, String),
}

impl Block {
    fn parse_bold(s: &mut Chars) -> Block {
        let mut blocks = Vec::new();

        while let Some(c) = s.next() {
            if c == '*' && s.as_str().starts_with('*') {
                let _ = s.next();
                break;
            } else {
                blocks.push(Block::Text(c))
            }
        }

        Block::Bold(blocks)
    }

    fn parse_italic(s: &mut Chars) -> Block {
        let mut blocks = Vec::new();

        for c in s {
            if c == '*' {
                break;
            } else {
                blocks.push(Block::Text(c))
            }
        }

        Block::Italic(blocks)
    }

    fn parse_code(s: &mut Chars) -> Block {
        let mut blocks = Vec::new();

        for c in s {
            if c == '`' {
                break;
            } else {
                blocks.push(Block::Text(c))
            }
        }

        Block::Code(blocks)
    }

    #[allow(clippy::while_let_on_iterator)]
    fn parse_link(s: &mut Chars) -> Block {
        let mut blocks = Vec::new();
        let mut link = String::new();

        while let Some(c) = s.next() {
            match c {
                ']' => break,
                _ => blocks.push(Block::Text(c)),
            }
        }

        if s.as_str().starts_with('(') {
            while let Some(c) = s.next() {
                match c {
                    '(' => (),
                    ')' => break,
                    _ => link.push(c),
                }
            }
        }

        Block::Link(blocks, link)
    }

    fn parse(s: &mut Chars) -> Vec<Block> {
        let mut blocks = Vec::new();

        while let Some(c) = s.next() {
            match c {
                '*' => {
                    if s.as_str().starts_with('*') {
                        let _ = s.next();
                        blocks.push(Block::parse_bold(s));
                    } else {
                        blocks.push(Block::parse_italic(s));
                    }
                }
                '`' => blocks.push(Block::parse_code(s)),
                '[' => blocks.push(Block::parse_link(s)),
                _ => blocks.push(Block::Text(c)),
            }
        }

        blocks
    }

    fn draw<W: IntoRawMode>(
        &self,
        to: &mut RawTerminal<W>,
        path: &PathBuf,
        next: &mut Vec<PathBuf>,
        next_i: usize,
    ) -> std::io::Result<usize> {
        let mut count = 0;

        match *self {
            Block::Text(c) => {
                count += to.write(&[c as u8])?;
            }
            Block::Bold(ref blocks) => {
                write!(to, "{}", style::Bold)?;
                for block in blocks.iter() {
                    count += block.draw(to, path, next, next_i)?;
                }
                write!(to, "{}", style::NoBold)?;
            }
            Block::Italic(ref blocks) => {
                write!(to, "{}", style::Italic)?;
                for block in blocks.iter() {
                    count += block.draw(to, path, next, next_i)?;
                }
                write!(to, "{}", style::NoItalic)?;
            }
            Block::Code(ref blocks) => {
                write!(to, "{}", color::Bg(color::AnsiValue::grayscale(6)))?;
                for block in blocks.iter() {
                    count += block.draw(to, path, next, next_i)?;
                }
                write!(to, "{}", color::Bg(color::Reset))?;
            }
            Block::Link(ref blocks, ref link) => {
                let highlight = next.len() == next_i;

                if link.starts_with('/') {
                    next.push(PathBuf::from(link));
                } else {
                    let mut next_path = path.clone();
                    next_path.pop();

                    for part in Path::new(&link).iter() {
                        match part.to_str().unwrap() {
                            "." => {}
                            ".." => {
                                while next_path.ends_with(".") && next_path.pop() {}
                                if next_path.ends_with("..") || !next_path.pop() {
                                    next_path.push(part);
                                }
                            }
                            _ => next_path.push(part),
                        }
                    }

                    next.push(next_path);
                }

                write!(to, "{}", style::Underline)?;
                if highlight {
                    write!(to, "{}", style::Invert)?;
                }
                for block in blocks.iter() {
                    count += block.draw(to, path, next, next_i)?;
                }
                write!(to, "{}", style::NoUnderline)?;
                if highlight {
                    write!(to, "{}", style::NoInvert)?;
                }
            }
        }

        Ok(count)
    }
}

struct Buffer {
    lines: Vec<Vec<Block>>,
    y_off: u16,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            lines: Vec::new(),
            y_off: 0,
        }
    }

    fn read(&mut self, from: &mut dyn Read) -> std::io::Result<usize> {
        let mut tmp = String::new();
        let res = from.read_to_string(&mut tmp)?;

        self.lines.append(
            &mut tmp
                .as_str()
                .split('\n')
                .map(|x| Block::parse(&mut x.chars()))
                .collect(),
        );

        Ok(res)
    }

    #[allow(clippy::explicit_counter_loop)]
    fn draw<W: IntoRawMode>(
        &self,
        to: &mut RawTerminal<W>,
        path: &PathBuf,
        next: &mut Vec<PathBuf>,
        next_i: usize,
        w: u16,
        h: u16,
    ) -> std::io::Result<usize> {
        write!(to, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1))?;

        let mut y = 0;
        let mut count = 0;

        let lines = if self.lines.len() <= h as usize {
            &self.lines
        } else if self.y_off as usize >= self.lines.len() {
            &self.lines[0..0]
        } else {
            &self.lines[self.y_off as usize..]
        };

        for line in lines {
            for block in line.iter() {
                let x = block.draw(to, path, next, next_i)?;
                count += x;
                if x >= w as usize {
                    break;
                }
            }

            y += 1;
            write!(to, "{}", cursor::Goto(1, y + 1))?;
            if y >= h {
                break;
            }
        }

        Ok(count)
    }

    fn scroll_up(&mut self) {
        if self.y_off > 0 {
            self.y_off -= 1;
        }
    }

    fn scroll_down(&mut self, height: u16) {
        if ((self.y_off + height) as usize) <= self.lines.len() {
            self.y_off += 1;
        }
    }
}

fn run<W: IntoRawMode>(
    mut path: PathBuf,
    file: &mut dyn Read,
    controls: &mut dyn Read,
    stdout: &mut W,
) -> std::io::Result<()> {
    let mut stdout = stdout.into_raw_mode()?;

    let (w, h) = {
        let (w, h) = terminal_size()?;
        (w as u16, h as u16)
    };

    let mut next = Vec::new();
    let mut next_i = 0;

    let mut buffer = Buffer::new();
    buffer.read(file)?;

    buffer.draw(&mut stdout, &path, &mut next, next_i, w, h - 1)?;

    write!(
        stdout,
        "{}{}{} Press q to exit.{}",
        cursor::Goto(1, h),
        style::Invert,
        path.display(),
        style::NoInvert
    )?;

    stdout.flush()?;

    for c in controls.keys() {
        match c.unwrap() {
            Key::Char('q') => {
                write!(
                    stdout,
                    "{}{}{}",
                    clear::All,
                    style::Reset,
                    cursor::Goto(1, 1)
                )?;
                break;
            }
            Key::Char('b') => {
                for _i in 1..h {
                    buffer.scroll_up()
                }
            }
            Key::Char(' ') => {
                for _i in 1..h {
                    buffer.scroll_down(h)
                }
            }
            Key::Up | Key::Char('k') => buffer.scroll_up(),
            Key::Down | Key::Char('j') => buffer.scroll_down(h),
            Key::Char('\t') => {
                next_i += 1;
                if next_i >= next.len() {
                    next_i = 0;
                }
            }
            Key::Char('\r') | Key::Char('\n') => {
                if let Some(next_path) = next.get(next_i) {
                    if let Ok(mut next_file) = File::open(&next_path) {
                        path = next_path.clone();
                        buffer = Buffer::new();
                        buffer.read(&mut next_file)?;
                        next_i = 0;
                    }
                }
            }
            _ => {}
        }

        next = Vec::new();

        buffer.draw(&mut stdout, &path, &mut next, next_i, w, h - 1)?;

        write!(
            stdout,
            "{}{}{} Press q to exit.{}",
            cursor::Goto(1, h),
            style::Invert,
            path.display(),
            style::NoInvert
        )?;

        stdout.flush()?;
    }

    Ok(())
}
