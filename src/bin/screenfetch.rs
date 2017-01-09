#![deny(warnings)]

extern crate raw_cpuid;
extern crate syscall;

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    let user = env::var("USER").unwrap_or(String::new());
    let mut hostname = String::new();
    if let Ok(mut file) = File::open("/etc/hostname") {
        let _ = file.read_to_string(&mut hostname);
    }

    let mut uptime = 0;

    let mut ts = syscall::TimeSpec::default();
    if syscall::clock_gettime(syscall::CLOCK_MONOTONIC, &mut ts).is_ok() {
        uptime = ts.tv_sec;
    }

    let mut width = 0;
    let mut height = 0;
    if let Ok(display_name) = env::var("DISPLAY") {
        if let Ok(display) = syscall::open(&display_name, syscall::O_STAT) {
            let mut buf: [u8; 4096] = [0; 4096];
            if let Ok(count) = syscall::fpath(display, &mut buf) {
                let path = unsafe { String::from_utf8_unchecked(Vec::from(&buf[..count])) };
                let res = path.split(":").nth(1).unwrap_or("");
                width = res.split("/").nth(1).unwrap_or("").parse::<i32>().unwrap_or(0);
                height = res.split("/").nth(2).unwrap_or("").parse::<i32>().unwrap_or(0);
            }
            let _ = syscall::close(display);
        }
    }

    let mut cpu = String::new();
    {
        let cpuid = raw_cpuid::CpuId::new();
        if let Some(info) = cpuid.get_extended_function_info() {
            if let Some(brand) = info.processor_brand_string() {
                cpu = brand.to_string();
            }
        }
    }

    let mut ram = String::new();
    {
        let mut stat = syscall::StatVfs::default();
        if let Ok(fd) = syscall::open("memory:", syscall::O_STAT) {
            if syscall::fstatvfs(fd, &mut stat).is_ok() {
                let size = stat.f_blocks * stat.f_bsize as u64;
                let used = (stat.f_blocks - stat.f_bfree) * stat.f_bsize as u64;

                ram = format!("{}MB / {}MB", (used + 1048575)/1048576, (size + 1048575)/1048576);
            }
            let _ = syscall::close(fd);
        }
    }

    let mut string = String::new();
    string.push_str(&format!("\x1B[1;38;5;75m                `.-/+NMN+-.`                   \x1B[0m\x1B[1;38;5;75m{}\x1B[0m@\x1B[1;38;5;75m{}\x1B[0m\n", user, hostname.trim()));
    string.push_str("\x1B[1;38;5;75m           `:+oo+/-.-yds--/+oo+:`              \x1B[0m\x1B[1;38;5;75mOS:\x1B[0m redox-os\n");
    string.push_str("\x1B[1;38;5;75m        `/ss/++::/+o++++o+/:```:ss/`           \x1B[0m\x1B[1;38;5;75mKernel:\x1B[0m redox\n");
    string.push_str(&format!("\x1B[1;38;5;75m        `/ss/++::/+o++++o+/:```:ss/`           \x1B[0m\x1B[1;38;5;75mUptime:\x1B[0m {}s\n", uptime));
    string.push_str("\x1B[1;38;5;75m      `+h+``oMMN+.````````.:+syyy:/h+`         \x1B[0m\x1B[1;38;5;75mShell:\x1B[0m ion\n");
    string.push_str(&format!("\x1B[1;38;5;75m     /h/+mmm/://:+oo+//+oo+:. hNNh.`/h/        \x1B[0m\x1B[1;38;5;75mResolution:\x1B[0m {}x{}\n", width, height));
    string.push_str("\x1B[1;38;5;75m    oy` ydds`/s+:`        `:+s/-.+Ndd-so       \x1B[0m\x1B[1;38;5;75mDE:\x1B[0m orbital\n");
    string.push_str("\x1B[1;38;5;75m   os `yo  /y:                :y/.dmM- so      \x1B[0m\x1B[1;38;5;75mWM:\x1B[0m orbital\n");
    string.push_str("\x1B[1;38;5;75m  :h  s+  os`   \x1B[0m smhhhyyy/  \x1B[1;38;5;75m   `so  +s  h:     \x1B[0m\x1B[1;38;5;75mFont:\x1B[0m unifont\n");
    string.push_str(&format!("\x1B[1;38;5;75m  m. -h  /h     \x1B[0m yM    .oM+ \x1B[1;38;5;75m     h/  h- .m     \x1B[0m\x1B[1;38;5;75mCPU:\x1B[0m {}\n", cpu));
    string.push_str(&format!("\x1B[1;38;5;75m  N  s+  d.     \x1B[0m yM     -Ms \x1B[1;38;5;75m     .d  +s  m     \x1B[0m\x1B[1;38;5;75mRAM:\x1B[0m {}\n", ram));
    string.push_str("\x1B[1;38;5;75m  h  y/  M      \x1B[0m yM :+sydy` \x1B[1;38;5;75m      M  /y  h     \x1B[0m\n");
    string.push_str("\x1B[1;38;5;75m  M  oo  y/     \x1B[0m yM .yNy.   \x1B[1;38;5;75m     /y  oo  M     \x1B[0m\n");
    string.push_str("\x1B[1;38;5;75m  y/ `m` .d.    \x1B[0m yM   :md-  \x1B[1;38;5;75m    .d.:hNy /y     \x1B[0m\n");
    string.push_str("\x1B[1;38;5;75m  .d` :h:--h:   \x1B[0m +s    `ss` \x1B[1;38;5;75m   :h- oMNh`d.     \x1B[0m\n");
    string.push_str("\x1B[1;38;5;75m   :d.-MMN:.oo:              :oo.+sd+..d:      \x1B[0m\n");
    string.push_str("\x1B[1;38;5;75m    -d//oyy////so/:oyo..ydhos/. +MMM::d-       \x1B[0m\n");
    string.push_str("\x1B[1;38;5;75m     `sy- yMMN. `./MMMo+dNm/ ./ss-./ys`        \x1B[0m\n");
    string.push_str("\x1B[1;38;5;75m       .ss/++:+oo+//:-..:+ooo+-``:ss.          \x1B[0m\n");
    string.push_str("\x1B[1;38;5;75m         `:ss/-` `.--::--.` `-/ss:`            \x1B[0m\n");
    string.push_str("\x1B[1;38;5;75m             ./oooooooooooooo/.                \x1B[0m\n");
    io::stdout().write(string.as_bytes()).unwrap();
}
