//! The digest a proof bundle is tied to, as `scripts/proof-digest.sh` prints it:
//! every staged file outside proof/, by path, mode and content.

use std::io;
use std::path::Path;

use crate::{git, sha256};

/// Returns `sha256:<hex>` over `git ls-files --stage -z` for everything outside proof/.
///
/// # Errors
/// When git fails.
pub fn proof_digest(repo: &Path) -> io::Result<String> {
    // -z prints raw, NUL-separated paths, so core.quotePath and other display
    // settings cannot change the digest.
    let listing = git(
        repo,
        &[
            "ls-files",
            "--stage",
            "-z",
            "--full-name",
            "--",
            ":/",
            ":(top,exclude)proof",
        ],
    )?;
    Ok(format!("sha256:{}", sha256::hex(&listing)))
}
