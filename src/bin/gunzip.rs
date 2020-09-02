extern crate extra;
extern crate libflate;

use extra::option::OptionalExt;
use libflate::gzip::Decoder;
use std::io::Write;
use std::{env, fs, io, process};

fn main() {
    let mut stderr = io::stderr();

    let mut keep = false;
    let mut files = Vec::new();
    for arg in env::args().skip(1) {
        if arg == "-k" {
            keep = true;
        } else {
            files.push(arg)
        }
    }

    if files.is_empty() {
        writeln!(stderr, "gunzip: no files provided").unwrap();
        process::exit(1);
    }

    for arg in files {
        if arg.ends_with(".gz") {
            {
                let input = fs::File::open(&arg).try(&mut stderr);
                let mut decoder = Decoder::new(input).try(&mut stderr);

                let mut output = fs::File::create(&arg.trim_end_matches(".gz")).try(&mut stderr);
                io::copy(&mut decoder, &mut output).try(&mut stderr);

                output.flush().try(&mut stderr);
            }
            if !keep {
                fs::remove_file(&arg).try(&mut stderr);
            }
        } else {
            writeln!(stderr, "gunzip: {}: unknown suffix", arg).unwrap();
            process::exit(2);
        }
    }
}
