extern crate termion;

use std::io::{self, Write};
use termion::cursor::{self, DetectCursorPos};
use termion::raw::IntoRawMode;

fn main() {
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    write!(stdout, "{}", cursor::Save).unwrap();
    stdout.flush().unwrap();

    write!(stdout, "{}", cursor::Goto(999, 999)).unwrap();
    stdout.flush().unwrap();

    let (w, h) = stdout.cursor_pos().unwrap();

    write!(stdout, "{}", cursor::Restore).unwrap();
    stdout.flush().unwrap();

    drop(stdout);

    println!("export COLUMNS={}", w);
    println!("export LINES={}", h);
}
