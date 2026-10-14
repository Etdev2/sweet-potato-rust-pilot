//! The rust-pilot lane's tooling, run as `cargo xtask <task>`. It replaces the
//! repository's own shell scripts. The proof tools copied from the control room
//! (`scripts/check-proof.sh`, `scripts/proof-digest.sh`, `scripts/write-proof.sh`)
//! stay as pinned shell copies; `prove` and `test-proof-tools` call them.

pub mod digest;
pub mod pinned;
pub mod proof_tools;
pub mod prove;
pub mod secrets;
pub mod sha256;
pub mod verify;

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root: the parent of this crate's directory.
#[must_use]
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Runs `git -C <repo> <args>` and returns its stdout, failing on a non-zero exit.
///
/// # Errors
/// When git can't start or exits non-zero.
pub fn git(repo: &Path, args: &[&str]) -> io::Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(io::Error::other(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}
