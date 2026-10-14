//! `cargo xtask <task>`: the lane's tooling. Run `cargo xtask` for the list.

use std::env;
use std::io;
use std::process::ExitCode;

use xtask::{digest, pinned, proof_tools, prove, repo_root, secrets, verify};

const USAGE: &str = "usage: cargo xtask <task>

tasks:
  verify                  run the release binary through the feature map; PASS / FAIL / INCONCLUSIVE
  prove <owner/repo#n>    run every lane check through scripts/write-proof.sh into proof/
  proof-digest [repo]     print the digest a proof bundle is tied to
  check-pinned-scripts [repo]
                          check the control-room copies against scripts/control-room-pin.txt
  check-secret-files [repo]
                          fail when a committed file is named like a secret
  test-proof-tools        prove the copied proof tools reject known-bad bundles";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let root = repo_root();
    let repo_arg = |index: usize| {
        args.get(index)
            .map_or_else(|| root.clone(), std::path::PathBuf::from)
    };
    let result: io::Result<u8> = match args.first().map(String::as_str) {
        Some("verify") => Ok(verify::run(&root)),
        Some("prove") => {
            if let Some(ticket) = args.get(1) {
                prove::run(&root, ticket)
            } else {
                eprintln!("usage: cargo xtask prove <owner/repo#issue>");
                Ok(2)
            }
        }
        Some("proof-digest") => digest::proof_digest(&repo_arg(1)).map(|digest| {
            println!("{digest}");
            0
        }),
        Some("check-pinned-scripts") => pinned::run(&repo_arg(1)),
        Some("check-secret-files") => secrets::run(&repo_arg(1)),
        Some("test-proof-tools") => proof_tools::run(&root),
        _ => {
            eprintln!("{USAGE}");
            Ok(2)
        }
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}
