use std::env;
use std::process::ExitCode;

use rust_pilot::TrustStage;

const USAGE: &str = "usage: rust-pilot <trust-stage>";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let [name] = args.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };

    match name.parse::<TrustStage>() {
        Ok(stage) => {
            println!("{stage}: {}", stage.builder_authority());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
