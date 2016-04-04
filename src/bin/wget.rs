use std::env;
use std::fs::File;
use std::io::{stderr, Read, Write};
use std::process;
use std::str;

fn main() {
    if let Some(url) = env::args().nth(1) {
        let (scheme, reference) = url.split_at(url.find(':').unwrap_or(0));
        if scheme == "http" {
            let mut parts = reference.split('/').skip(2); //skip first two slashes
            let remote = parts.next().unwrap_or("");
            let path = parts.next().unwrap_or("/");

            let mut remote_parts = remote.split(':');
            let host = remote_parts.next().unwrap_or("127.0.0.1");
            let port = remote_parts.next().unwrap_or("80");

            let tcp = format!("tcp:{}:{}", host, port);
            let mut stream = File::open(tcp).unwrap();

            let request = format!("GET {} HTTP/1.0\r\n\r\n", path);
            stream.write(request.as_bytes()).unwrap();
            stream.flush().unwrap();

            let mut bytes = [0; 8192];
            let count = stream.read(&mut bytes).unwrap();
            println!("{}", unsafe { str::from_utf8_unchecked(&bytes[.. count]) });
        } else {
            write!(stderr(), "wget: unknown scheme '{}'\n", scheme).unwrap();
            process::exit(1);
        }
    } else {
        write!(stderr(), "wget: http://host:port/path\n").unwrap();
        process::exit(1);
    }
}
