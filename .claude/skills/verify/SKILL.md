---
name: verify
description: Prove a change to rust-pilot works by running the real release binary, then write the proof bundle. Use before claiming any ticket is done.
---

# Verify rust-pilot

rust-pilot is a command-line tool. Its real artifact is the release binary, run the way an operator runs it.

## Steps

1. Stage your whole change: `git add -A`.
2. Run `scripts/prove.sh <owner/repo#issue>`. It runs every lane check, runs `scripts/verify.sh` against the release binary, and writes `proof/`.
3. Read `proof/verify.txt`. If the result is not `PASS`, fix the cause and go back to step 1. Never edit `proof/` by hand.
4. Commit `proof/` with your change.

If you changed behavior, add a case to `scripts/verify.sh` and a row to the feature map in the same pull request. A feature with no case is unverified.

## Feature map

| Feature | How an operator uses it | Case in `scripts/verify.sh` |
| --- | --- | --- |
| Stage lookup | `rust-pilot stage-1-supervised` prints what a builder may do | `stage lookup: *` |
| Unknown stage | A misspelled stage exits 2 and lists the valid names | `unknown stage is rejected` |
| Usage | No argument exits 2 with usage | `missing argument prints usage` |
