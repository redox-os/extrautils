extern crate extra;
extern crate libflate;

use extra::option::OptionalExt;
use libflate::gzip::Encoder;
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
        eprintln!("gzip: no files provided");
        process::exit(1);
    }

    for arg in files {
        {
            let output = fs::File::create(&format!("{}.gz", &arg)).try(&mut stderr);
            let mut encoder = Encoder::new(output).try(&mut stderr);

            let mut input = fs::File::open(&arg).try(&mut stderr);
            io::copy(&mut input, &mut encoder).try(&mut stderr);

            let mut encoded = encoder.finish().into_result().try(&mut stderr);
            encoded.flush().try(&mut stderr);
        }
        if !keep {
            fs::remove_file(&arg).try(&mut stderr);
        }
    }
}
