//! Proves the copied proof tools reject the bundles that the pilot's earlier
//! check-proof.sh accepted (review of rust-pilot PR #1), and that the drift check
//! catches a changed copy. Each case runs in a throwaway copy of this checkout.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{git, pinned};

/// A scratch directory removed on drop.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn scratch() -> io::Result<Scratch> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let dir =
        std::env::temp_dir().join(format!("xtask-proof-tools-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir)?;
    Ok(Scratch(dir))
}

fn commit(repo: &Path, message: &str) -> io::Result<()> {
    git(repo, &["add", "-A"])?;
    git(
        repo,
        &[
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-qm",
            message,
        ],
    )?;
    Ok(())
}

/// Copies every tracked and unignored file outside proof/ into a fresh repository.
fn new_repo(root: &Path, work: &Path, name: &str) -> io::Result<PathBuf> {
    let repo = work.join(name);
    fs::create_dir_all(&repo)?;
    let listing = git(root, &["ls-files", "-co", "--exclude-standard", "-z"])?;
    for entry in listing.split(|byte| *byte == 0) {
        let path = String::from_utf8_lossy(entry);
        if path.is_empty() || path.starts_with("proof/") {
            continue;
        }
        let from = root.join(path.as_ref());
        if !from.exists() {
            continue;
        }
        let to = repo.join(path.as_ref());
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&from, &to)?;
    }
    git(&repo, &["init", "-q"])?;
    commit(&repo, "base")?;
    Ok(repo)
}

/// Writes a valid committed bundle with the copied writer.
fn prove(repo: &Path) -> io::Result<()> {
    let status = Command::new("bash")
        .arg(repo.join("scripts/write-proof.sh"))
        .arg("--repo")
        .arg(repo)
        .args(["--ticket", "Etdev2/sweet-potato#24"])
        .args([
            "--fixture",
            "a throwaway copy",
            "--shows",
            "The output of true.",
        ])
        .args(["--", "true"])
        .stdout(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(io::Error::other(
            "write-proof.sh failed on a throwaway copy",
        ));
    }
    commit(repo, "Add proof")
}

fn edit_manifest(repo: &Path, filter: &str) -> io::Result<()> {
    let manifest = repo.join("proof/manifest.json");
    let output = Command::new("jq").arg(filter).arg(&manifest).output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!("jq {filter} failed")));
    }
    let tmp = repo.join("m.tmp");
    fs::write(&tmp, output.stdout)?;
    fs::rename(tmp, manifest)
}

fn check_proof(repo: &Path) -> io::Result<(i32, String)> {
    let output = Command::new("bash")
        .arg(repo.join("scripts/check-proof.sh"))
        .arg(repo)
        .output()?;
    let mut text = output.stdout;
    text.extend_from_slice(&output.stderr);
    Ok((
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&text).into_owned(),
    ))
}

#[derive(Default)]
struct Tally {
    failures: u32,
}

impl Tally {
    fn record(&mut self, name: &str, ok: bool, code: i32, out: &str) {
        if ok {
            println!("ok   {name}");
            return;
        }
        println!("FAIL {name} (exit {code})");
        for line in out.trim_end_matches('\n').lines() {
            println!("     {line}");
        }
        self.failures += 1;
    }

    fn expect(
        &mut self,
        name: &str,
        want_pass: bool,
        repo: &Path,
        pattern: &str,
    ) -> io::Result<()> {
        let (code, out) = check_proof(repo)?;
        let ok = (code == 0) == want_pass && out.contains(pattern);
        self.record(name, ok, code, &out);
        Ok(())
    }
}

/// Runs every case and prints the results. Returns the exit code.
///
/// # Errors
/// When a throwaway repository can't be set up.
pub fn run(root: &Path) -> io::Result<u8> {
    let work = scratch()?;
    let work = work.0.as_path();
    let mut tally = Tally::default();

    let repo = new_repo(root, work, "valid")?;
    prove(&repo)?;
    tally.expect("a bundle from the copied writer passes", true, &repo, "")?;

    let repo = new_repo(root, work, "pattern")?;
    prove(&repo)?;
    edit_manifest(&repo, r#".evidence[0].path = "proof/v*.txt""#)?;
    commit(&repo, "Name evidence by pattern")?;
    tally.expect(
        "evidence named by a pattern fails",
        false,
        &repo,
        "is not committed",
    )?;

    let repo = new_repo(root, work, "directory")?;
    prove(&repo)?;
    edit_manifest(&repo, r#".evidence[0].path = "proof/""#)?;
    commit(&repo, "Name a directory as evidence")?;
    tally.expect(
        "evidence naming a directory fails",
        false,
        &repo,
        "is not committed",
    )?;

    // The review's case: an unstaged ticket edit must not change what is checked.
    let repo = new_repo(root, work, "unstaged-ticket")?;
    prove(&repo)?;
    edit_manifest(&repo, r#".ticket = "Etdev2/sweet-potato#999""#)?;
    let (code, out) = check_proof(&repo)?;
    tally.record(
        "an unstaged manifest edit is ignored; the committed ticket is checked",
        code == 0 && out.contains("PASS for Etdev2/sweet-potato#24.") && !out.contains("#999"),
        code,
        &out,
    );

    // The same gap where it matters: a committed FAIL can't pass by an uncommitted edit.
    let repo = new_repo(root, work, "uncommitted-pass")?;
    prove(&repo)?;
    edit_manifest(&repo, r#".result = "FAIL""#)?;
    commit(&repo, "Commit a FAIL manifest")?;
    edit_manifest(&repo, r#".result = "PASS""#)?;
    tally.expect(
        "an uncommitted edit can't turn a committed FAIL into PASS",
        false,
        &repo,
        "Verification reported FAIL",
    )?;

    let repo = new_repo(root, work, "drift")?;
    OpenOptions::new()
        .append(true)
        .open(repo.join("scripts/check-proof.sh"))?
        .write_all(b"\n")?;
    let report = pinned::check(&repo)?;
    let out = report.lines.join("\n");
    tally.record(
        "the drift check catches a one-byte change to a copied script",
        !report.passed && out.contains("scripts/check-proof.sh differs from the reviewed copy"),
        i32::from(!report.passed),
        &out,
    );

    if tally.failures > 0 {
        println!("{} proof-tool case(s) failed.", tally.failures);
        return Ok(1);
    }
    println!("All proof-tool cases behave as specified.");
    Ok(0)
}
