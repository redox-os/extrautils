use std::fs::File;
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::process::Command;

use system::syscall::*;
use system::scheme::Packet;

extern crate system;

fn main() {
    let command = "ps";
    match Command::new(command).spawn_supervise() {
        Ok(child) => {
            println!("Spawned {}", child.id());
            match sys_supervise(child.id() as usize) {
                Ok(fd) => {
                    println!("Supervised {}", fd);
                    let mut file = unsafe { File::from_raw_fd(fd) };
                    loop {
                        let mut packet = Packet::default();
                        match file.read(&mut packet) {
                            Ok(_) => {
                                match packet.a {
                                    SYS_BRK => println!("brk(0x{:X}) = 0x{:X}", packet.b, packet.id),
                                    SYS_EXIT => {
                                        println!("exit({}) = {}", packet.b, packet.id);
                                        break;
                                    },
                                    SYS_READ => println!("read({}, 0x{:X}, {}) = {}", packet.b, packet.c, packet.d, packet.id),
                                    SYS_WRITE => println!("write({}, 0x{:X}, {}) = {}", packet.b, packet.c, packet.d, packet.id),
                                    SYS_OPEN => println!("open(0x{:X}, {}, {}) = {}", packet.b, packet.c, packet.d, packet.id),
                                    SYS_CLOSE => println!("close({}) = {}", packet.b, packet.id),
                                    SYS_LSEEK => println!("lseek({}, {}, {}) = {}", packet.b, packet.c as isize, packet.d, packet.id),
                                    _ => println!("{}({}, {}, {}) = {}", packet.a, packet.b, packet.c, packet.d, packet.id)
                                }
                                /*
                                match file.write(&packet) {
                                    Ok(_) => {
                                        println!("Allowed syscall");

                                        match file.read(&mut packet) {
                                            Ok(_) => {
                                                println!("Returning {:#?}", packet);
                                            },
                                            Err(err) => {

                                            }
                                    },
                                    Err(err) => {
                                        println!("Write Request Error: {}", err);
                                    }
                                }
                                */
                            },
                            Err(err) => {
                                println!("Read Request Error: {}", err);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    println!("Failed to supervise {}: {}", child.id(), err);
                }
            }
        },
        Err(err) => {
            println!("Failed to spawn {}: {}", command, err);
        }
    }
}
