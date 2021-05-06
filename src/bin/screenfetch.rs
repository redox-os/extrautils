#[cfg(target_arch = "x86_64")]
extern crate raw_cpuid;
extern crate syscall;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// std::fmt::Write conflicts with std::io::Write, hence the alias
use std::fmt::Write as FmtWrite;

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 3600;
const SECONDS_PER_DAY: i64 = 86400;

fn main() {
    let user = env::var("USER").unwrap_or_default();
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
            fmt_result = write!(
                &mut uptime_str,
                "{}d {}h {}m {}s",
                uptime_days, uptime_hours, uptime_mins, uptime_secs
            );
        } else if uptime_hours > 0 {
            fmt_result = write!(
                &mut uptime_str,
                "{}h {}m {}s",
                uptime_hours, uptime_mins, uptime_secs
            );
        } else if uptime_mins > 0 {
            fmt_result = write!(&mut uptime_str, "{}m {}s", uptime_mins, uptime_secs);
        } else {
            fmt_result = write!(&mut uptime_str, "{}s", uptime_secs);
        }

        if let Err(err) = fmt_result {
            eprintln!("error: couldn't parse uptime, {}", err);
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
                let res = path.split(':').nth(1).unwrap_or("");
                width = res
                    .split('/')
                    .nth(1)
                    .unwrap_or("")
                    .parse::<i32>()
                    .unwrap_or(0);
                height = res
                    .split('/')
                    .nth(2)
                    .unwrap_or("")
                    .parse::<i32>()
                    .unwrap_or(0);
            }
            let _ = syscall::close(display);
        }
    }

    let mut cpu = String::new();
    #[cfg(target_arch = "x86_64")]
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

                ram = format!(
                    "{}MB / {}MB",
                    (used + 1_048_575) / 1_048_576,
                    (size + 1_048_575) / 1_048_576
                );
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

    const S: &str = "\x1B[1;38;5;75m"; // blue start
    const E: &str = "\x1B[0m"; // end
    let right = [
        format!("{}{}{}@{}{}{}", S, user, E, S, hostname.trim(), E),
        format!("{}OS:         {}redox-os", S, E),
        format!("{}Kernel:     {}redox", S, E),
        format!("{}Uptime:     {}{}", S, E, uptime_str),
        format!("{}Shell:      {}{}", S, E, shell),
        format!("{}Resolution: {}{}x{}", S, E, width, height),
        format!("{}DE:         {}orbital", S, E),
        format!("{}WM:         {}orbital", S, E),
        format!("{}Font:       {}unifont", S, E),
        format!("{}CPU:        {}{}", S, E, cpu),
        format!("{}RAM:        {}{}", S, E, ram),
    ];

    for (i, line) in left.iter().enumerate() {
        print!("\x1B[1;38;5;75m{}  \x1B[0m", line);
        if let Some(r) = right.get(i) {
            print!("{}", r);
        }
        println!();
    }
}
