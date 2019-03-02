#![deny(warnings)]

extern crate raw_cpuid;
extern crate syscall;

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

// std::fmt::Write conflicts with std::io::Write, hence the alias
use std::fmt::Write as FmtWrite;

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 3600;
const SECONDS_PER_DAY: i64 = 86400;

fn main() {
    let user = env::var("USER").unwrap_or(String::new());
    let mut hostname = String::new();
    if let Ok(mut file) = File::open("file:/etc/hostname") {
        let _ = file.read_to_string(&mut hostname);
    }

    let mut uptime_str = String::new();

    let mut ts = syscall::TimeSpec::default();
    if syscall::clock_gettime(syscall::CLOCK_MONOTONIC, &mut ts).is_ok() {
        let uptime = ts.tv_sec;
        let uptime_secs = uptime % 60;
        let uptime_mins = (uptime / SECONDS_PER_MINUTE) % 60;
        let uptime_hours = (uptime / SECONDS_PER_HOUR) % 24;
        let uptime_days = (uptime / SECONDS_PER_DAY) % 365;

        let fmt_result;
        if uptime_days > 0 {
            fmt_result = write!(&mut uptime_str, "{}d {}h {}m {}s", uptime_days,
                                uptime_hours, uptime_mins, uptime_secs);
        } else if uptime_hours > 0 {
            fmt_result = write!(&mut uptime_str, "{}h {}m {}s", uptime_hours,
                                uptime_mins, uptime_secs);
        } else if uptime_mins > 0 {
            fmt_result = write!(&mut uptime_str, "{}m {}s", uptime_mins,
                                uptime_secs);
        } else {
            fmt_result = write!(&mut uptime_str, "{}s", uptime_secs);
        }

        if let Err(_) = fmt_result {
            // We probably don't want to panic! on this error
            println!("error: couldn't parse uptime");
        }
    }

    let mut shell = String::new();
    {
        if let Ok(shell_path) = env::var("SHELL") {
            if let Some(file_name) = Path::new(&shell_path).file_name() {
                shell = file_name.to_str().unwrap_or("").to_string();
            }
        }
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
                cpu = brand.trim().to_string();
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

    let left = [
        "              :+yMMMMy+:              ",
        "         .+dddNMMMMMMMMNddd+.         ",
        "       sydMMMMo/sMMMMs/oMMMMdys       ",
        "    .oMMMdso              osdMMMo.    ",
        "   .+MMd/`   -:::::::`      `/dMM+.   ",
        "  +dMMN.     NMMNNNMMdyo`     .NMMd+  ",
        "  yMMN       NM+   oomMMN       NMMy  ",
        " hNMd.       NM+     `oMN       .dMNh ",
        " dMMh        NM+     `oMN        hMMd ",
        " dMMh        NM+-oooodMMN        hMMd ",
        " dMMh        NM+/MMMMdhs`        hMMd ",
        " hNMd.       NM+`/mMMm+         .dMNh ",
        "  yMMN       NM+   oNMd+        NMMy  ",
        "  +dMMN.     NM+    omMMm     .NMMd+  ",
        "   .+MMd/`   --.     .---   `/dMM+.   ",
        "    .oMMMdso              osdMMMo.    ",
        "       sydMMMMo////////oMMMMdys       ",
        "         .+dddNMMMMMMMMNddd+.         ",
        "              :++++++++:              ",
    ];

    let right = [
        format!("\x1B[1;38;5;75m{}\x1B[0m@\x1B[1;38;5;75m{}\x1B[0m", user, hostname.trim()),
        format!("\x1B[1;38;5;75mOS:\x1B[0m redox-os"),
        format!("\x1B[1;38;5;75mKernel:\x1B[0m redox"),
        format!("\x1B[1;38;5;75mUptime:\x1B[0m {}", uptime_str),
        format!("\x1B[1;38;5;75mShell:\x1B[0m {}", shell),
        format!("\x1B[1;38;5;75mResolution:\x1B[0m {}x{}", width, height),
        format!("\x1B[1;38;5;75mDE:\x1B[0m orbital"),
        format!("\x1B[1;38;5;75mWM:\x1B[0m orbital"),
        format!("\x1B[1;38;5;75mFont:\x1B[0m unifont"),
        format!("\x1B[1;38;5;75mCPU:\x1B[0m {}", cpu),
        format!("\x1B[1;38;5;75mRAM:\x1B[0m {}", ram)
    ];

    let mut string = String::new();
    for i in 0..left.len() {
        string.push_str("\x1B[1;38;5;75m");
        string.push_str(left[i]);
        string.push_str("  \x1B[0m");
        if let Some(r) = right.get(i) {
            string.push_str(r);
        }
        string.push_str("\n");
    }

    io::stdout().write(string.as_bytes()).unwrap();
}
