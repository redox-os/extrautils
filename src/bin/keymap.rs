extern crate extra;
extern crate syscall;
extern crate byteorder;

use std::{io, str, env};
use std::io::{Read, Write, Seek};
use std::fs::File;
use std::process::exit;
use std::io::SeekFrom;
use byteorder::{BigEndian, ByteOrder};


use extra::option::OptionalExt;
use extra::io::WriteExt;

static MAN_PAGE: &'static str = /* @MANSTART{keymap} */ r#"
NAME
    keymap - change the keymap

SYNOPSIS
    keymap [-h | --help] [-l --list] [PATH]

DESCRIPTION
    Changes the keymap to that provided in PATH. The file format is similar
    to xmodmap but the right-hand side of the `=` can only describe printable
    characters. Look in /etc/keymaps for examples.
OPTIONS
    -h
    --help
        Print this manual page.
"#; /* @MANEND */

pub const N_MOD_COMBOS: usize = 4;
pub const N_SCANCODES: usize = 58;

fn main() {
    let mut args = env::args().skip(1);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let arg  = match args.next() {
        Some(arg) => arg,
        None => {
            // TODO Write explanation
            println!("Needs one argument.");
            exit(1);
        }
    };
    let path = if arg.starts_with("-") {
        match arg.as_str() {
            "-h" | "--help" => {
                stdout.writeln(MAN_PAGE.as_bytes()).try(&mut stderr);
                exit(0);
            },
            _ => {
                stderr.write(b"Unknown option: ").try(&mut stderr);
                stderr.write(arg.as_bytes()).try(&mut stderr);
                stderr.write(b"\n").try(&mut stderr);
                let _ = stderr.flush();
                exit(1);
            }
        }
    } else {
        arg
    };
    let mut scheme = File::open("ps2:keymap").expect("keymap: could not open scheme: {}");
    let mut file = File::open(path).expect("keymap: could not open file: {}");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("keymap: could not read file");

    // Parse file contents. Should maybe use nom...
    for (i, line) in contents.lines().enumerate() {
        if i >= N_SCANCODES {
            break;
        }
        let mut keycode = 0;
        let mut row_buf: [u8; 4*N_MOD_COMBOS] = [0; 4*N_MOD_COMBOS];
        let mut n_entries = 0;
        for (j, element) in line.split_whitespace().enumerate() {
            match j {
                0 => if element != "keycode" {
                    eprintln!("Keyword 'keycode' expected on line {}", i);
                    std::process::exit(1);
                },
                1 => if let Ok(n) = element.parse::<u8>() {
                    keycode = n;
                } else {
                    eprintln!("Second keyword on line {} should be an integer in the range 0..256.", i);
                    std::process::exit(1);
                },
                2 => if element != "=" {
                    eprintln!("Third keyword on line {} should be '='.", i);
                    std::process::exit(1);
                },
                j => {
                    n_entries += 1;
                    let j = j - 3;
                    let c: char = to_char(&element.as_bytes());
                    let u: u32  = u32::from(c);
                    BigEndian::write_u32(&mut row_buf[j*4..], u);
                }
            }
        }

        // Duplicate entries if fewer than N_MOD_COMBOS are specified
        if n_entries == 1 {
            let mut first_entry: [u8; 4] = [0; 4];
            first_entry.copy_from_slice(&row_buf[0..4]);
            row_buf[4..8].copy_from_slice(&first_entry);
            row_buf[8..12].copy_from_slice(&first_entry);
            row_buf[12..16].copy_from_slice(&first_entry);
        } else if n_entries == 2 {
            let mut first_entries: [u8; 8] = [0; 8];
            first_entries.copy_from_slice(&row_buf[0..8]);
            row_buf[8..16].copy_from_slice(&first_entries);
        }
        // Find the row for the right keycode
        scheme.seek(SeekFrom::Start(keycode as u64 * N_MOD_COMBOS as u64 * 4)).expect("Failed to seek");
        match scheme.write_all(&row_buf) {
            Ok(_) => {},
            Err(e) =>  {
                if let Some(syscall::EINVAL) = e.raw_os_error() {
                    println!("Invalid syntax in keymap file");
                } else {
                    println!("Error applying keymap: {}", e);
                }
                std::process::exit(1);
            }
        }
    }
}


/// Parse single character from text.
#[allow(unused)]
fn to_char(text: &[u8]) -> char {
    match text.len() {
        1 => text[0] as char,

        2 => match text[1] {
                // Explicit hex string with one digit
                b'0' ... b'9' | b'A' ... b'F' => {
                    u8::from_str_radix(str::from_utf8(&text[1..2]).unwrap_or("0"), 16).unwrap_or(0) as char
                }
                b'n' => '\n',
                b't' => '\t',
                // Quote, single quote or backslash, or some character I haven't yet thought about
                c => c as char,
            },
        3 => {
            u8::from_str_radix(str::from_utf8(&text[1..3]).unwrap_or("0"), 16).unwrap_or(0) as char
        }
        _ => 0 as char,
    }
}
