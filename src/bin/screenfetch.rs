#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern crate raw_cpuid;
extern crate libredox;

use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

// std::fmt::Write conflicts with std::io::Write, hence the alias
use std::fmt::Write as FmtWrite;

use libredox::Fd;

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 3600;
const SECONDS_PER_DAY: i64 = 86400;

const KIB: u64 = 1024;
const MIB: u64 = 1024 * KIB;
const GIB: u64 = 1024 * MIB;
const TIB: u64 = 1024 * GIB;


fn format_size(size: u64) -> String {
    if size >= 4 * TIB {
        format!("{:.1} TiB", size as f64 / TIB as f64)
    } else if size >= GIB {
        format!("{:.1} GiB", size as f64 / GIB as f64)
    } else if size >= MIB {
        format!("{:.1} MiB", size as f64 / MIB as f64)
    } else if size >= KIB {
        format!("{:.1} KiB", size as f64 / KIB as f64)
    } else {
        format!("{} B", size)
    }
}

fn main() {
    let user = env::var("USER").unwrap_or_default();
    let mut hostname = String::new();
    if let Ok(mut file) = File::open("/etc/hostname") {
        let _ = file.read_to_string(&mut hostname);
    }

    let os_pretty_name = match os_release::OsRelease::new() {
        Ok(ok) => ok.pretty_name,
        Err(err) => {
            eprintln!("error: failed to read /etc/os-release: {}", err);
            "(unknown release)".to_string()
        }
    };

    let mut kernel = String::new();
    match fs::read_to_string("/scheme/sys/uname") {
        Ok(uname) => for line in uname.lines() {
            if line.is_empty() {
                continue;
            }

            if ! kernel.is_empty() {
                kernel.push(' ');
            }

            kernel.push_str(line);
        },
        Err(err) => {
            eprintln!("error: failed to read /scheme/sys/uname: {}", err);
        }
    }

    let mut uptime_str = String::new();

    if let Ok(ts) = libredox::call::clock_gettime(libredox::flag::CLOCK_MONOTONIC) {
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
        if let Ok(display) = Fd::open(&display_name, libredox::flag::O_PATH, 0) {
            let mut buf: [u8; 4096] = [0; 4096];
            if let Ok(count) = display.fpath(&mut buf) {
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
        }
    }

    let mut cpu = String::new();
    #[cfg(target_arch = "x86")]
    {
        let cpuid = raw_cpuid::CpuId::with_cpuid_fn(|a, c| {
            let result = unsafe { core::arch::x86::__cpuid_count(a, c) };
            raw_cpuid::CpuIdResult {
                eax: result.eax,
                ebx: result.ebx,
                ecx: result.ecx,
                edx: result.edx,
            }
        });
        if let Some(brand) = cpuid.get_processor_brand_string() {
            cpu = brand.as_str().to_string();
        }
    }
    #[cfg(target_arch = "x86_64")]
    {
        let cpuid = raw_cpuid::CpuId::new();
        if let Some(brand) = cpuid.get_processor_brand_string() {
            cpu = brand.as_str().to_string();
        }
    }

    let mut ram = String::new();
    {
        if let Ok(fd) = Fd::open("/scheme/memory", libredox::flag::O_PATH, 0) {
            if let Ok(stat) = fd.statvfs() {
                let size = stat.f_blocks as u64 * stat.f_bsize as u64;
                let used = (stat.f_blocks as u64 - stat.f_bfree as u64) * stat.f_bsize as u64;

                ram = format!(
                    "{} / {} ({}%)",
                    format_size(used),
                    format_size(size),
                    used * 100 / size
                );
            }
        }
    }

    let mut disk = String::new();
    {
        if let Ok(fd) = Fd::open("/", libredox::flag::O_PATH, 0) {
            if let Ok(stat) = fd.statvfs() {
                let size = stat.f_blocks as u64 * stat.f_bsize as u64;
                let used = (stat.f_blocks as u64 - stat.f_bfree as u64) * stat.f_bsize as u64;

                disk = format!(
                    "{} / {} ({}%)",
                    format_size(used),
                    format_size(size),
                    used * 100 / size
                );
            }
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
        format!("{}OS:         {}{}", S, E, os_pretty_name),
        format!("{}Kernel:     {}{}", S, E, kernel),
        format!("{}Uptime:     {}{}", S, E, uptime_str),
        format!("{}Shell:      {}{}", S, E, shell),
        format!("{}Resolution: {}{}x{}", S, E, width, height),
        format!("{}DE:         {}orbital", S, E),
        format!("{}WM:         {}orbital", S, E),
        format!("{}CPU:        {}{}", S, E, cpu),
        format!("{}RAM:        {}{}", S, E, ram),
        format!("{}Disk:       {}{}", S, E, disk),
    ];

    for (i, line) in left.iter().enumerate() {
        print!("\x1B[1;38;5;75m{}  \x1B[0m", line);
        if let Some(r) = right.get(i) {
            print!("{}", r);
        }
        println!();
    }
}
