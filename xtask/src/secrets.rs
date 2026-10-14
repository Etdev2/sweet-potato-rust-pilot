//! Fails when a committed file is named like a secret: env files, private keys,
//! certificates or credential files. Content scanning is a separate decision.

use std::io;
use std::path::Path;

use crate::git;

/// Whether `path` matches the secret-name pattern the shell check used:
/// `(^|/)(\.env(\..+)?|id_(rsa|ed25519)[^/]*|credentials\.json)$|\.(pem|key|p12|pfx)$`,
/// minus `(^|/)\.env\.example$`.
#[must_use]
pub fn looks_secret(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    if name == ".env.example" {
        return false;
    }
    let by_extension = [".pem", ".key", ".p12", ".pfx"]
        .iter()
        .any(|ext| path.ends_with(ext));
    let by_name =
        name.starts_with("id_rsa") || name.starts_with("id_ed25519") || name == "credentials.json";
    // `.env(\..+)?$` after a slash can run across later slashes, so test every
    // component start, not just the last name.
    let env_file = component_starts(path).any(|rest| {
        rest == ".env"
            || rest
                .strip_prefix(".env.")
                .is_some_and(|tail| !tail.is_empty())
    });
    by_extension || by_name || env_file
}

fn component_starts(path: &str) -> impl Iterator<Item = &str> {
    std::iter::once(path).chain(
        path.match_indices('/')
            .filter_map(move |(index, _)| path.get(index + 1..)),
    )
}

/// Returns the committed paths that look like secrets.
///
/// # Errors
/// When git fails.
pub fn secret_files(repo: &Path) -> io::Result<Vec<String>> {
    let listing = git(repo, &["ls-files", "-z"])?;
    Ok(listing
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).into_owned())
        .filter(|path| looks_secret(path))
        .collect())
}

/// Runs the check, printing what the shell script printed. Returns the exit code.
///
/// # Errors
/// When git fails.
pub fn run(repo: &Path) -> io::Result<u8> {
    let found = secret_files(repo)?;
    if found.is_empty() {
        println!("No secret-looking files are committed.");
        return Ok(0);
    }
    for path in found {
        println!(
            "::error file={path}::{path} looks like a secret. Remove it from git, rotate whatever it held, and keep it in GitHub secrets instead."
        );
    }
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::looks_secret;

    #[test]
    fn flags_the_same_names_as_the_shell_pattern() {
        for path in [
            ".env",
            "app/.env",
            ".env.local",
            "a/.env.d/x",
            "id_rsa",
            "keys/id_ed25519.pub",
            "credentials.json",
            "x/server.pem",
            "tls.key",
            "c.p12",
            "c.pfx",
        ] {
            assert!(looks_secret(path), "{path}");
        }
        for path in [
            ".env.example",
            "a/.env.example",
            "src/env.rs",
            "my.env",
            ".env.",
            "notid_rsa",
            "credentials.json.bak",
            "key.txt",
            "Cargo.toml",
        ] {
            assert!(!looks_secret(path), "{path}");
        }
    }
}
