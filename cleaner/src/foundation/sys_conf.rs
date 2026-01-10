// this using spawn getconf using process::command that much slower rather than libc
// that use low-level api
//
// use std::path::PathBuf;
// use std::process::Command;
// pub const DARWIN_USER_CACHE_DIR: &str = "DARWIN_USER_CACHE_DIR";
// pub const DARWIN_USER_TEMP_DIR: &str = "DARWIN_USER_TEMP_DIR";

// pub fn sysconf_path(name: &str) -> Option<PathBuf> {
//     let out = Command::new("getconf").arg(name).output().ok()?;

//     if out.status.success() {
//         let s = String::from_utf8_lossy(&out.stdout)
//             .trim() // removes \n
//             .trim_end_matches('/') // removes trailing /
//             .to_string();

//         Some(PathBuf::from(s))
//     } else {
//         None
//     }
// }
// ==============
// this using libc query that much better performance with directly interaction
use libc::confstr;
use std::ffi::CStr;
use std::path::PathBuf;

pub const DARWIN_USER_CACHE_DIR: i32 = libc::_CS_DARWIN_USER_CACHE_DIR;
pub const DARWIN_USER_TEMP_DIR: i32 = libc::_CS_DARWIN_USER_TEMP_DIR;

pub fn sysconf_path(name: i32) -> Option<PathBuf> {
    unsafe {
        // First call: get required buffer size
        let len = confstr(name, std::ptr::null_mut(), 0);
        if len == 0 {
            return None;
        }

        let mut buf = vec![0u8; len as usize];

        // Second call: fill buffer
        let written = confstr(name, buf.as_mut_ptr() as *mut _, len);
        if written == 0 {
            return None;
        }

        let s = CStr::from_ptr(buf.as_ptr() as *const _)
            .to_string_lossy()
            .trim() // remove newline if any
            .trim_end_matches('/') // match bash sed
            .to_string();

        Some(PathBuf::from(s))
    }
}
