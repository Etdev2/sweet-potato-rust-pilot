//! The Rust tools give the same answers as the shell they replaced, and as the
//! pinned control-room copies they sit beside.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xtask::{digest, pinned, repo_root, secrets};

type TestResult = Result<(), Box<dyn Error>>;

const SECRET_PATTERN: &str =
    r"(^|/)(\.env(\..+)?|id_(rsa|ed25519)[^/]*|credentials\.json)$|\.(pem|key|p12|pfx)$";

fn temp_repo(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let dir = std::env::temp_dir().join(format!("xtask-parity-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    xtask::git(&dir, &["init", "-q"])?;
    Ok(dir)
}

fn shell_digest(repo: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("bash")
        .arg(repo_root().join("scripts/proof-digest.sh"))
        .arg(repo)
        .output()?;
    assert!(output.status.success());
    Ok(String::from_utf8(output.stdout)?.trim_end().to_owned())
}

#[test]
fn digest_matches_the_pinned_script_on_this_checkout() -> TestResult {
    let root = repo_root();
    assert_eq!(digest::proof_digest(&root)?, shell_digest(&root)?);
    Ok(())
}

#[test]
fn digest_matches_the_pinned_script_on_awkward_paths() -> TestResult {
    let repo = temp_repo("digest")?;
    fs::create_dir_all(repo.join("proof"))?;
    fs::create_dir_all(repo.join("dir with space"))?;
    fs::write(repo.join("proof/ignored.txt"), "excluded")?;
    fs::write(repo.join("dir with space/naïve\tname.txt"), "x")?;
    fs::write(repo.join("line\nbreak"), "y")?;
    xtask::git(&repo, &["add", "-A"])?;
    let rust = digest::proof_digest(&repo)?;
    assert_eq!(rust, shell_digest(&repo)?);
    // proof/ is outside the digest.
    fs::write(repo.join("proof/ignored.txt"), "changed")?;
    xtask::git(&repo, &["add", "-A"])?;
    assert_eq!(digest::proof_digest(&repo)?, rust);
    fs::remove_dir_all(repo)?;
    Ok(())
}

#[test]
fn digest_of_an_empty_index_is_the_empty_hash() -> TestResult {
    let repo = temp_repo("empty")?;
    assert_eq!(
        digest::proof_digest(&repo)?,
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    fs::remove_dir_all(repo)?;
    Ok(())
}

#[test]
fn secret_check_matches_the_shell_pattern() -> TestResult {
    let repo = temp_repo("secrets")?;
    let names = [
        ".env",
        "a/.env.prod",
        ".env.example",
        "b/.env.example",
        "my.env",
        "id_rsa.pub",
        "k/id_ed25519",
        "credentials.json",
        "x.pem",
        "x.key",
        "x.keys",
        "c.p12",
        "c.pfx",
        "src/lib.rs",
        ".env.d/inner",
    ];
    for name in names {
        let path = repo.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, "")?;
    }
    xtask::git(&repo, &["add", "-Af"])?;
    let shell = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "git ls-files | grep -E '{SECRET_PATTERN}' | grep -vE '(^|/)\\.env\\.example$' || true"
        ))
        .current_dir(&repo)
        .output()?;
    let mut want: Vec<String> = String::from_utf8(shell.stdout)?
        .lines()
        .map(str::to_owned)
        .collect();
    let mut got = secrets::secret_files(&repo)?;
    want.sort();
    got.sort();
    assert_eq!(got, want);
    assert!(!got.is_empty());
    fs::remove_dir_all(repo)?;
    Ok(())
}

#[test]
fn this_checkout_has_no_secret_files_and_matching_pins() -> TestResult {
    let root = repo_root();
    assert!(secrets::secret_files(&root)?.is_empty());
    let report = pinned::check(&root)?;
    assert!(report.passed, "{:?}", report.lines);
    assert_eq!(
        report.lines,
        ["3 copied scripts match the pinned control-room commit."]
    );
    Ok(())
}

#[test]
fn pin_check_flags_missing_and_non_executable_copies() -> TestResult {
    let repo = temp_repo("pins")?;
    fs::create_dir_all(repo.join("scripts"))?;
    fs::write(repo.join("scripts/a.sh"), "echo a\n")?;
    let hash = String::from_utf8(xtask::git(&repo, &["hash-object", "scripts/a.sh"])?)?;
    fs::write(
        repo.join("scripts/control-room-pin.txt"),
        format!(
            "# comment\n\n100755 blob {}\tscripts/a.sh\n100644 blob {}\tscripts/gone.sh\n",
            hash.trim(),
            hash.trim()
        ),
    )?;
    let report = pinned::check(&repo)?;
    assert!(!report.passed);
    assert_eq!(report.lines.len(), 2, "{:?}", report.lines);
    assert!(report.lines[0].contains("scripts/a.sh must be executable"));
    assert!(report.lines[1].contains("scripts/gone.sh is missing"));

    fs::write(repo.join("scripts/control-room-pin.txt"), "# nothing\n")?;
    let report = pinned::check(&repo)?;
    assert!(!report.passed);
    assert!(report.lines[0].contains("The pin lists no scripts."));
    fs::remove_dir_all(repo)?;
    Ok(())
}
