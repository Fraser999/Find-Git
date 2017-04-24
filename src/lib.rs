//! A cross-platform library which returns the full path to the Git binary if it can be found.

#![forbid(warnings)]
#![warn(missing_copy_implementations, trivial_casts, trivial_numeric_casts, unsafe_code,
        unused_extern_crates, unused_import_braces, unused_qualifications, unused_results,
        variant_size_differences)]
#![cfg_attr(feature="cargo-clippy", deny(clippy, clippy_pedantic))]
#![cfg_attr(feature="cargo-clippy", allow(missing_docs_in_private_items))]

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate winreg;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

#[cfg(windows)]
const LOCATE_COMMAND: &'static str = "where";
#[cfg(not(windows))]
const LOCATE_COMMAND: &'static str = "which";

fn git_ran_ok<S: AsRef<OsStr>>(binary_path: S) -> bool {
    Command::new(binary_path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn try_git_with_no_path() -> Option<::std::path::PathBuf> {
    if let Ok(output) = Command::new(LOCATE_COMMAND).arg("git").output() {
        let git = str::from_utf8(&output.stdout)
            .expect(&format!("Non-UTF8 output when running `{} git`.", LOCATE_COMMAND))
            .trim()
            .lines()
            .next()
            .expect(&format!("Should have had at least one line of text when running `{} git`.",
                             LOCATE_COMMAND));
        if git_ran_ok(&git) {
            return Some(PathBuf::from(git));
        }
    }
    None
}

#[cfg(windows)]
mod find_git {
    use super::{git_ran_ok, try_git_with_no_path};
    use std::ffi::OsStr;
    use std::io::Result;
    use std::path::PathBuf;
    use winapi::HKEY;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

    fn try_full_path_to_git<P: AsRef<OsStr>>(predefined_key: HKEY,
                                             subkey_path: P,
                                             value: P)
                                             -> Option<PathBuf> {
        let root = RegKey::predef(predefined_key);
        if let Ok(subkey) = root.open_subkey_with_flags(subkey_path, KEY_READ) {
            let subkey_value: Result<String> =
                subkey.get_value(value);
            if let Ok(install_path) = subkey_value {
                let binary_path = PathBuf::from(&install_path).join("bin").join("git.exe");
                if git_ran_ok(binary_path.as_os_str()) {
                    return Some(binary_path);
                }
            }
        }
        None
    }

    /// Tries to find Git.  On Windows, if it is not currently available in `%PATH%`, tries using
    /// several known registry values.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn git_path() -> Option<PathBuf> {
        const SUBKEY_RECENT: &'static str = "Software\\GitForWindows";
        const SUBKEY_32_BIT: &'static str =
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Git_is1";
        const SUBKEY_64_BIT: &'static str =
            "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Git_is1";
        try_git_with_no_path()
            .or_else(|| try_full_path_to_git(HKEY_LOCAL_MACHINE, SUBKEY_RECENT, "InstallPath"))
            .or_else(|| try_full_path_to_git(HKEY_CURRENT_USER, SUBKEY_32_BIT, "InstallLocation"))
            .or_else(|| try_full_path_to_git(HKEY_CURRENT_USER, SUBKEY_64_BIT, "InstallLocation"))
            .or_else(|| try_full_path_to_git(HKEY_LOCAL_MACHINE, SUBKEY_32_BIT, "InstallLocation"))
            .or_else(|| try_full_path_to_git(HKEY_LOCAL_MACHINE, SUBKEY_64_BIT, "InstallLocation"))
    }
}

#[cfg(not(windows))]
mod find_git {
    pub fn git_path() -> Option<::std::path::PathBuf> { super::try_git_with_no_path() }
}

pub use find_git::git_path;
