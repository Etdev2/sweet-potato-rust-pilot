# rust-pilot

The pilot project for Sweet Potato's `rust-service` lane. The control room is [Etdev2/sweet-potato](https://github.com/Etdev2/sweet-potato); its `CONTEXT.md` defines the words used here.

## Before you say a ticket is done

Use the `verify` skill in `.claude/skills/verify/`. Every pull request commits a passing proof bundle in `proof/`, made by `cargo xtask prove`.

## Rules

| Rule | Enforced by |
| --- | --- |
| Code is formatted with rustfmt. | Hard check: `cargo fmt --check` in CI |
| No compiler or Clippy warnings. `clippy::all` and `clippy::pedantic` are errors. | Hard check: `[lints]` in `Cargo.toml`, with `CARGO_BUILD_WARNINGS=deny` in CI |
| No `unsafe`. | Hard check: `unsafe_code = "forbid"`, which no inner attribute can override |
| No `unwrap`, `expect`, `panic!`, `todo!` or `dbg!` outside tests. | Hard check: Clippy lints in `Cargo.toml` |
| Arithmetic overflow panics in release builds as well. | Hard check: `overflow-checks = true` in `[profile.release]` |
| The toolchain and dependency versions are fixed. | Hard check: `rust-toolchain.toml`, and `--locked` in CI |
| Every pull request carries fresh, passing proof. | Hard check: `scripts/check-proof.sh` (pinned control-room copy) in CI |
| The proof scripts copied from the control room match its reviewed commit. | Hard check: `cargo xtask check-pinned-scripts` against `scripts/control-room-pin.txt`, and `cargo xtask test-proof-tools`, in CI |
| No secret-looking file (env files, private keys, credential files) is committed. | Hard check: `.gitignore`, plus `cargo xtask check-secret-files` in CI |
| Tokens and keys live in GitHub secrets, never in code. | Review; file names alone can't catch a key pasted into code |
| A lint exception uses `#[expect(lint, reason = "...")]`, never `#[allow]`, and fails once the code no longer needs it. | Hard check: `allow_attributes` and `allow_attributes_without_reason` in `Cargo.toml` |
| An exception's reason explains why the safer form can't work. | Review |
| Model states with enums that carry each state's data. Parse external input once at the edge. | Review |
| Adding a dependency records why the standard library can't do the job. | Review |

Don't weaken a lint, a check or the toolchain pin to make a change pass. Fix the code, or propose an ADR in the control room.

## Commands

```text
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
cargo xtask verify                     # real-artifact check, prints PASS / FAIL / INCONCLUSIVE
cargo xtask prove Etdev2/<repo>#<n>    # all of the above, written to proof/
cargo xtask                            # lists every task
```

The lane's tooling is Rust, in the `xtask/` crate (`cargo xtask` is an alias in `.cargo/config.toml`). The only shell left is the three proof tools copied byte for byte from the control room and pinned in `scripts/control-room-pin.txt`: `scripts/check-proof.sh`, `scripts/proof-digest.sh` and `scripts/write-proof.sh`. Don't rewrite or edit those here; change them in the control room. `cargo xtask proof-digest` prints the same digest as `scripts/proof-digest.sh`, and a test holds them equal.
