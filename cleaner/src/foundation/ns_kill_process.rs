use anyhow::Result;
use std::process::Command;

// use libc::kill;
// use nix::unistd::Pid;
// use std::io;

// pub fn kill_pids(pids: &[i32]) -> io::Result<()> {
//     for &pid in pids {
//         // SIGTERM
//         let res = unsafe { kill(pid, libc::SIGTERM) };
//         if res != 0 {
//             eprintln!("Failed to kill PID {}: {}", pid, io::Error::last_os_error());
//         }
//     }
//     Ok(())
// }

// use rustix::process::kill;
// use rustix::process::Pid;
// use rustix::signal::Signal;

// fn kill_pids(pids: &str) {
//     for pid_str in pids.split_whitespace() {
//         if let Ok(pid) = pid_str.parse::<i32>() {
//             let _ = kill(Pid::from_raw(pid), Signal::kill());
//         }
//     }
// }

/// Kill the given PIDs using AppleScript
pub fn kill_pids(pids: &str) -> Result<()> {
    let script = format!(
        r#"do shell script "kill {} 2>/dev/null" with administrator privileges"#,
        pids
    );

    Command::new("osascript").arg("-e").arg(script).status()?;

    Ok(())
}
