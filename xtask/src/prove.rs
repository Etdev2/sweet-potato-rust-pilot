//! Runs every lane check and the real-artifact verification through the control
//! room's proof writer (scripts/write-proof.sh, ADR 0005 in Etdev2/sweet-potato),
//! which writes proof/. Stage your change by name first; then commit proof/
//! alongside it.

use std::io;
use std::path::Path;
use std::process::Command;

use crate::git;

/// The commands recorded in every proof bundle, in order.
pub const COMMANDS: &[&str] = &[
    "cargo xtask check-pinned-scripts",
    "cargo xtask test-proof-tools",
    "cargo fmt --all --check",
    "cargo clippy --workspace --all-targets --all-features --locked",
    "cargo test --workspace --all-features --locked",
    "cargo test --workspace --doc --locked",
    "cargo xtask verify",
];

/// Whether anything outside proof/ is unstaged or untracked.
///
/// # Errors
/// When git fails.
pub fn has_unstaged_work(root: &Path) -> io::Result<bool> {
    let diff = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--quiet", "--", ".", ":!proof"])
        .status()?;
    let untracked = git(
        root,
        &[
            "ls-files",
            "--others",
            "--exclude-standard",
            "--",
            ".",
            ":!proof",
        ],
    )?;
    Ok(!diff.success() || !untracked.is_empty())
}

/// Writes the proof bundle for `ticket`. Returns the writer's exit code.
///
/// # Errors
/// When git or the writer can't start.
pub fn run(root: &Path, ticket: &str) -> io::Result<u8> {
    // write-proof.sh stages everything outside proof/ with `git add -A`. Refusing
    // here first keeps this repo's rule: only files staged by name reach a commit.
    if has_unstaged_work(root)? {
        eprintln!(
            "Unstaged or untracked files exist outside proof/. Stage the files your change needs by name, and delete scratch files, so the proof matches the code it ran against."
        );
        return Ok(2);
    }
    let status = Command::new(root.join("scripts/write-proof.sh"))
        .current_dir(root)
        .env("CARGO_BUILD_WARNINGS", "deny")
        .args(["--ticket", ticket])
        .args(["--verification", "cargo xtask verify"])
        .args(["--fixture", "the feature map in .claude/skills/verify/SKILL.md"])
        .args([
            "--shows",
            "Every lane check's output and exit code, then each feature in the map run through the release binary, ending in RESULT: PASS.",
        ])
        .arg("--")
        .args(COMMANDS)
        .status()?;
    Ok(status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .unwrap_or(1))
}
