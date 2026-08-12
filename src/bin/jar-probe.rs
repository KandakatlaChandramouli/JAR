use std::fs;

fn main() {
    println!("=== JAR PROBE DIAGNOSTIC REPORT ===");

    println!("[PID] Self PID: {}", std::process::id());
    println!("[UID] EUID: {}", unsafe { libc::geteuid() });
    println!("[GID] EGID: {}", unsafe { libc::getegid() });

    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("Seccomp:")
                || line.starts_with("Seccomp_filters:")
                || line.starts_with("CapInh:")
                || line.starts_with("CapPrm:")
                || line.starts_with("CapEff:")
                || line.starts_with("CapBnd:")
                || line.starts_with("CapAmb:")
                || line.starts_with("NSpid:")
            {
                println!("[STATUS] {}", line);
            }
        }
    } else {
        println!("[STATUS] Could not read /proc/self/status");
    }

    if let Ok(uid_map) = fs::read_to_string("/proc/self/uid_map") {
        print!("[UID_MAP] {}", uid_map);
    } else {
        println!("[UID_MAP] Could not read /proc/self/uid_map");
    }

    if let Ok(cgroup) = fs::read_to_string("/proc/self/cgroup") {
        print!("[CGROUP] {}", cgroup);
    } else {
        println!("[CGROUP] Could not read /proc/self/cgroup");
    }

    match std::env::current_dir() {
        Ok(path) => println!("[PWD] Current directory: {}", path.display()),
        Err(e) => println!("[PWD] getcwd failed: {}", e),
    }

    match unsafe { libc::fork() } {
        -1 => println!("[FORK] Fork failed: {}", std::io::Error::last_os_error()),
        0 => {
            unsafe { libc::_exit(0) };
        }
        pid => {
            let mut status = 0;
            unsafe { libc::waitpid(pid, &mut status, 0) };
            println!("[FORK] Fork succeeded (child PID: {})", pid);
        }
    }

    println!("===================================");
}
