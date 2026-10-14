//! Runs the release binary the way an operator does and checks each feature in
//! the feature map (.claude/skills/verify/SKILL.md). Prints a transcript, then
//! exits 0 for PASS, 1 for FAIL and 2 for INCONCLUSIVE.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One feature-map case.
struct Case {
    feature: &'static str,
    want_code: i32,
    want_text: &'static str,
    args: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        feature: "stage lookup: unverified",
        want_code: 0,
        want_text: "no builder runs",
        args: &["stage-0-unverified"],
    },
    Case {
        feature: "stage lookup: supervised",
        want_code: 0,
        want_text: "Et inspects every step",
        args: &["stage-1-supervised"],
    },
    Case {
        feature: "stage lookup: proven",
        want_code: 0,
        want_text: "Et still merges",
        args: &["stage-2-proven"],
    },
    Case {
        feature: "unknown stage is rejected",
        want_code: 2,
        want_text: "use stage-0-unverified, stage-1-supervised or stage-2-proven",
        args: &["stage-3-automerge"],
    },
    Case {
        feature: "missing argument prints usage",
        want_code: 2,
        want_text: "usage: rust-pilot <trust-stage>",
        args: &[],
    },
];

/// Builds the release binary in `root` and runs every case. Returns the exit code.
#[must_use]
pub fn run(root: &Path) -> u8 {
    let built = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--locked",
            "--quiet",
            "--package",
            "rust-pilot",
        ])
        .current_dir(root)
        .status()
        .is_ok_and(|status| status.success());
    if !built {
        println!("RESULT: INCONCLUSIVE (the release binary did not build)");
        return 2;
    }
    let target = env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);
    let bin = root.join(target).join("release/rust-pilot");

    let mut failed = false;
    for case in CASES {
        let (out, code) = match Command::new(&bin).args(case.args).output() {
            Ok(output) => {
                let mut text = output.stdout;
                text.extend_from_slice(&output.stderr);
                (
                    String::from_utf8_lossy(&text)
                        .trim_end_matches('\n')
                        .to_owned(),
                    output.status.code().unwrap_or(-1),
                )
            }
            Err(error) => (error.to_string(), 127),
        };
        println!("$ rust-pilot {}", case.args.join(" "));
        println!("{out}");
        println!("[exit {code}]");
        if code == case.want_code && out.contains(case.want_text) {
            println!("ok   {}", case.feature);
        } else {
            println!(
                "FAIL {}: wanted exit {} and output containing \"{}\"",
                case.feature, case.want_code, case.want_text
            );
            failed = true;
        }
        println!();
    }

    if failed {
        println!("RESULT: FAIL");
        1
    } else {
        println!("RESULT: PASS");
        0
    }
}
