use std::error::Error;
use std::io;
use std::process::{Command, Output};

type TestResult = Result<(), Box<dyn Error>>;

fn run(args: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_rust-pilot"))
        .args(args)
        .output()
}

#[test]
fn each_stage_reports_its_builder_authority() -> TestResult {
    let cases = [
        ("stage-0-unverified", "no builder runs"),
        ("stage-1-supervised", "Et inspects every step"),
        ("stage-2-proven", "Et still merges"),
    ];
    for (stage, authority) in cases {
        let output = run(&[stage])?;
        let stdout = String::from_utf8(output.stdout)?;
        assert!(output.status.success(), "{stage} failed");
        assert!(stdout.starts_with(stage), "{stdout}");
        assert!(stdout.contains(authority), "{stdout}");
    }
    Ok(())
}

#[test]
fn unknown_stage_is_rejected_with_the_valid_names() -> TestResult {
    let output = run(&["stage-3-automerge"])?;
    let stderr = String::from_utf8(output.stderr)?;
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr.contains("unknown trust stage \"stage-3-automerge\""),
        "{stderr}"
    );
    assert!(stderr.contains("stage-0-unverified"), "{stderr}");
    Ok(())
}

#[test]
fn wrong_argument_count_prints_usage() -> TestResult {
    for args in [&[][..], &["stage-0-unverified", "extra"][..]] {
        let output = run(args)?;
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8(output.stderr)?.contains("usage:"));
    }
    Ok(())
}
