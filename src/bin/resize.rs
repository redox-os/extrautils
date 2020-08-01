extern crate extra;
extern crate termion;

use extra::option::OptionalExt;
use std::io::{self, Write};
use termion::cursor::{self, DetectCursorPos};
use termion::raw::IntoRawMode;

fn main() {
    let mut stderr = io::stderr();

    let terminal = termion::get_tty().try(&mut stderr);
    let mut terminal = terminal.into_raw_mode().try(&mut stderr);

    write!(terminal, "{}", cursor::Save).unwrap();
    terminal.flush().unwrap();

    write!(terminal, "{}", cursor::Goto(999, 999)).unwrap();
    terminal.flush().unwrap();

    let (w, h) = terminal.cursor_pos().unwrap();

    write!(terminal, "{}", cursor::Restore).unwrap();
    terminal.flush().unwrap();

    drop(terminal);

    println!("export COLUMNS={};", w);
    println!("export LINES={};", h);
}
