use std::env;
use std::io::{stderr, Read, Write};
use std::net::TcpStream;
use std::process;
use std::str;

fn main() {
    if let Some(url) = env::args().nth(1) {
        let (scheme, reference) = url.split_at(url.find(':').unwrap_or(0));
        if scheme == "http" {
            let mut parts = reference.split('/').skip(2); //skip first two slashes
            let remote = parts.next().unwrap_or("");
            let mut path = parts.next().unwrap_or("").to_string();
            for part in parts {
                path.push('/');
                path.push_str(part);
            }

            write!(stderr(), "* Connecting to {}\n", remote).unwrap();

            let mut stream = TcpStream::connect(&remote).unwrap();

            write!(stderr(), "* Requesting {}\n", path).unwrap();

            let request = format!("GET /{} HTTP/1.1\r\nHost: {}\r\n\r\n", path, env::args().nth(2).unwrap_or(remote.to_string()));
            stream.write(request.as_bytes()).unwrap();
            stream.flush().unwrap();

            write!(stderr(), "* Waiting for response\n").unwrap();

            let mut response = [0; 65536];
            let count = stream.read(&mut response).unwrap();

            let mut headers = true;
            for line in unsafe { str::from_utf8_unchecked(&response[.. count]) }.lines() {
                if headers {
                    if line.is_empty() {
                        headers = false;
                    } else {
                        write!(stderr(), "> {}\n", line).unwrap();
                    }
                } else {
                    println!("{}", line);
                }
            }
        } else {
            write!(stderr(), "wget: unknown scheme '{}'\n", scheme).unwrap();
            process::exit(1);
        }
    } else {
        write!(stderr(), "wget: http://host:port/path\n").unwrap();
        process::exit(1);
    }
}
