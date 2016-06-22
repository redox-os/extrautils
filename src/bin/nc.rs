extern crate system;

use std::env;
use std::io::{stdin, Read, Write, Result};
use std::str;
use std::sync::Arc;
use std::thread;

use std::cell::UnsafeCell;
use std::fs::File;

/// Redox domain socket
pub struct Socket {
    file: UnsafeCell<File>
}

unsafe impl Send for Socket {}
unsafe impl Sync for Socket {}

impl Socket {
    pub fn open(path: &str) -> Result<Socket> {
        let file = try!(File::open(path));
        Ok(Socket {
            file: UnsafeCell::new(file)
        })
    }

    pub fn receive(&self, buf: &mut [u8]) -> Result<usize> {
        unsafe { (*self.file.get()).read(buf) }
    }

    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        unsafe { (*self.file.get()).write(buf) }
    }
}


fn main() {
    let mut args = env::args().skip(1);
    if let Some(path) = args.next() {
        let socket_write = Arc::new(Socket::open(&path).unwrap());
        let socket_read = socket_write.clone();

        println!("USING {}", path);

        thread::spawn(move || {
            loop {
                let mut buffer = [0; 65536];
                let count = socket_read.receive(&mut buffer).unwrap();
                print!("{}", unsafe { str::from_utf8_unchecked(&buffer[..count]) });
            }
        });

        loop {
            let mut buffer = [0; 65536];
            let count = stdin().read(&mut buffer).unwrap();
            socket_write.send(&buffer[..count]).unwrap();
        }
    }
}
