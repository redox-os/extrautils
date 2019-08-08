extern crate tar;
extern crate tree_magic;
extern crate lzma;
extern crate libflate;
extern crate filetime;
extern crate bzip2;

use std::{env, process};
use std::io::{stdin, stdout, stderr, copy, Error, ErrorKind, Result, Read, Write, BufReader};
use std::fs::{self, File};
use std::os::unix::fs::{OpenOptionsExt, symlink};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use tar::{Archive, Builder, EntryType};
use lzma::LzmaReader;
use libflate::gzip::Decoder as GzipDecoder;
use bzip2::read::BzDecoder;
use filetime::FileTime;

fn create_inner<T: Write>(input: &str, ar: &mut Builder<T>) -> Result<()> {
    if try!(fs::metadata(input)).is_dir() {
        for entry_result in try!(fs::read_dir(input)) {
            let entry = try!(entry_result);
            if try!(fs::metadata(entry.path())).is_dir() {
                try!(create_inner(entry.path().to_str().unwrap(), ar));
            } else {
                println!("{}", entry.path().display());
                try!(ar.append_path(entry.path()));
            }
        }
    } else {
        println!("{}", input);
        try!(ar.append_path(input));
    }

    Ok(())
}

fn create(input: &str, tar: &str) -> Result<()> {
    if tar == "-" {
        create_inner(input, &mut Builder::new(stdout()))
    } else {
        create_inner(input, &mut Builder::new(try!(File::create(tar))))
    }
}

fn list_inner<T: Read>(ar: &mut Archive<T>) -> Result<()> {
    for entry_result in try!(ar.entries()) {
        let entry = try!(entry_result);
        let path = try!(entry.path());
        println!("{}", path.display());
    }

    Ok(())
}

fn list(tar: &str) -> Result<()> {
    if tar == "-" {
        list_inner(&mut Archive::new(stdin()))
    } else {
        list_inner(&mut Archive::new(try!(File::open(tar))))
    }
}

fn create_symlink(link: PathBuf, target: &Path) -> Result<()> {
    //delete existing file to make way for symlink
    if link.exists() {
        fs::remove_file(link.clone()).expect(&format!("could not overwrite: {:?}", link));
    }
    symlink(target, link)
}

fn extract_inner<T: Read>(ar: &mut Archive<T>, verbose: bool, strip: usize) -> Result<()> {
    for entry_result in try!(ar.entries()) {
        let mut entry = try!(entry_result);

        let path = {
            let path = try!(entry.path());
            let mut components = path.components();
            for _ in 0..strip {
                components.next();
            }
            components.as_path().to_path_buf()
        };

        if path == Path::new("") {
            continue;
        }

        match entry.header().entry_type() {
            EntryType::Regular => {
                {
                    let mut file = {
                        if let Some(parent) = path.parent() {
                            try!(fs::create_dir_all(parent));
                        }
                        try!(
                            fs::OpenOptions::new()
                                .read(true)
                                .write(true)
                                .truncate(true)
                                .create(true)
                                .mode(entry.header().mode().unwrap_or(644))
                                .open(&path)
                        )
                    };
                    try!(copy(&mut entry, &mut file));
                }
                if let Ok(mtime) = entry.header().mtime() {
                    let mtime = FileTime::from_seconds_since_1970(mtime, 0);
                    try!(filetime::set_file_times(&path, mtime, mtime));
                }
            },
            EntryType::Directory => {
                try!(fs::create_dir_all(&path));
            },
            EntryType::Symlink => {
                if let Some(target) = entry.link_name().expect(&format!("Can't parse symlink target for: {:?}", path)) {
                    try!(create_symlink(path, &target))
                }
            },
            other => {
                panic!("Unsupported entry type {:?}", other);
            }
        }

        if verbose {
            println!("{}", entry.path()?.display());
        }
    }

    Ok(())
}

fn extract(tar: &Path, verbose: bool, strip: usize) -> Result<()> {
    if tar == Path::new("-") {
        extract_inner(&mut Archive::new(stdin()), verbose, strip)
    } else {
        let mime = tree_magic::from_filepath(Path::new(&tar));
        let file = BufReader::new(File::open(tar)?);
        if mime == "application/x-xz" {
            extract_inner(&mut Archive::new(LzmaReader::new_decompressor(file)
                                            .map_err(|e| Error::new(ErrorKind::Other, e))?),
                                            verbose, strip)
        } else if mime == "application/gzip" {
            extract_inner(&mut Archive::new(GzipDecoder::new(file)
                                            .map_err(|e| Error::new(ErrorKind::Other, e))?),
                                            verbose, strip)
        } else if mime == "application/x-bzip" {
            extract_inner(&mut Archive::new(BzDecoder::new(file)), verbose, strip)
        } else {
            extract_inner(&mut Archive::new(file), verbose, strip)
        }
    }
}

fn main() {
    let mut args = env::args().skip(1);
    if let Some(op) = args.next() {
        match op.as_str() {
            "c" => if let Some(input) = args.next() {
                if let Err(err) = create(&input, "-") {
                    write!(stderr(), "tar: create: failed: {}\n", err).unwrap();
                    process::exit(1);
                }
            } else {
                write!(stderr(), "tar: create: no input specified: {}\n", op).unwrap();
                process::exit(1);
            },
            "cf" => if let Some(tar) = args.next() {
                if let Some(input) = args.next() {
                    if let Err(err) = create(&input, &tar) {
                        write!(stderr(), "tar: create: failed: {}\n", err).unwrap();
                        process::exit(1);
                    }
                } else {
                    write!(stderr(), "tar: create: no input specified: {}\n", op).unwrap();
                    process::exit(1);
                }
            } else {
                write!(stderr(), "tar: create: no tarfile specified: {}\n", op).unwrap();
                process::exit(1);
            },
            "t" | "tf" => {
                let tar = args.next().unwrap_or("-".to_string());
                if let Err(err) = list(&tar) {
                    write!(stderr(), "tar: list: failed: {}\n", err).unwrap();
                    process::exit(1);
                }
            },
            "x" | "xf" | "xvf" => {
                let mut tar = None;
                let mut strip = 0;
                while let Some(arg) = args.next() {
                    if arg == "-C" || arg == "--directory" {
                        env::set_current_dir(args.next().expect(&format!("{} requires path", arg))).unwrap();
                    } else if arg.starts_with("--directory=") {
                        env::set_current_dir(&arg[12..]).unwrap();
                    } else if arg.starts_with("--strip-components") {
                        let num = args.next().expect("--strip-components requires an integer");
                        strip = usize::from_str(&num).expect("--strip-components requires an integer");
                    } else if arg.starts_with("--strip-components=") {
                        strip = usize::from_str(&arg[19..]).expect("--strip-components requires an integer");
                    } else if tar.is_none() {
                        let mut path = env::current_dir().unwrap();
                        path.push(arg);
                        tar = Some(path);
                    }
                }
                let tar = tar.unwrap_or(PathBuf::from("-"));

                let verbose = op.contains('v');
                if let Err(err) = extract(&tar, verbose, strip) {
                    write!(stderr(), "tar: extract: failed: {}\n", err).unwrap();
                    process::exit(1);
                }
            },
            _ => {
                write!(stderr(), "tar: {}: unknown operation\n", op).unwrap();
                write!(stderr(), "tar: need to specify c[f] (create), t[f] (list), or x[f] (extract)\n").unwrap();
                process::exit(1);
            }
        }
    } else {
        write!(stderr(), "tar: no operation\n").unwrap();
        write!(stderr(), "tar: need to specify cf (create), tf (list), or xf (extract)\n").unwrap();
        process::exit(1);
    }
}
