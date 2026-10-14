//! Fails when a script copied from the control room differs from the reviewed
//! commit recorded in scripts/control-room-pin.txt. The control room is private,
//! so this compares git blob hashes offline instead of fetching it.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::git;

/// What the check found: the lines to print and whether it passed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub lines: Vec<String>,
    pub passed: bool,
}

/// Checks every pinned copy under `repo`.
///
/// # Errors
/// When the pin file can't be read or git can't hash a file.
pub fn check(repo: &Path) -> io::Result<Report> {
    let pin = fs::read_to_string(repo.join("scripts/control-room-pin.txt"))?;
    let mut lines = Vec::new();
    let mut failed = false;
    let mut checked = 0usize;

    for line in pin.lines() {
        // `read -r mode type hash path`: three whitespace-separated fields,
        // then the rest of the line is the path.
        let mut fields = line.trim_start().splitn(4, [' ', '\t']);
        let mode = fields.next().unwrap_or("");
        if mode.is_empty() || mode.starts_with('#') {
            continue;
        }
        let kind = fields.next().unwrap_or("");
        let hash = fields.next().unwrap_or("");
        let path = fields.next().unwrap_or("").trim();
        checked += 1;

        let file = repo.join(path);
        let regular = fs::symlink_metadata(&file).is_ok_and(|meta| meta.file_type().is_file());
        if kind != "blob" || path.is_empty() || !regular {
            lines.push(format!(
                "::error file={path}::{path} is missing or not a regular file. Copy it from the commit in scripts/control-room-pin.txt."
            ));
            failed = true;
            continue;
        }
        let actual = git(repo, &["hash-object", "--", path])?;
        if String::from_utf8_lossy(&actual).trim() != hash {
            lines.push(format!(
                "::error file={path}::{path} differs from the reviewed copy pinned in scripts/control-room-pin.txt. Change it in the control room, then update the copy and the pin."
            ));
            failed = true;
        }
        let executable =
            fs::metadata(&file).is_ok_and(|meta| meta.permissions().mode() & 0o111 != 0);
        if mode == "100755" && !executable {
            lines.push(format!(
                "::error file={path}::{path} must be executable, as it is at the pinned commit."
            ));
            failed = true;
        }
    }

    if checked == 0 {
        lines.push(
            "::error file=scripts/control-room-pin.txt::The pin lists no scripts.".to_owned(),
        );
        return Ok(Report {
            lines,
            passed: false,
        });
    }
    if !failed {
        lines.push(format!(
            "{checked} copied scripts match the pinned control-room commit."
        ));
    }
    Ok(Report {
        lines,
        passed: !failed,
    })
}

/// Runs the check and prints its report. Returns the exit code.
///
/// # Errors
/// As [`check`].
pub fn run(repo: &Path) -> io::Result<u8> {
    let report = check(repo)?;
    for line in &report.lines {
        println!("{line}");
    }
    Ok(u8::from(!report.passed))
}
