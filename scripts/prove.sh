#!/usr/bin/env bash
set -euo pipefail

# Runs every lane check and the real-artifact verification through the control
# room's proof writer (scripts/write-proof.sh, ADR 0005 in Etdev2/sweet-potato),
# which writes proof/. Stage your change by name first; then commit proof/
# alongside it.
#
# usage: scripts/prove.sh <owner/repo#issue>
ticket="${1:?usage: scripts/prove.sh <owner/repo#issue>}"
cd "$(dirname "$0")/.."

# write-proof.sh stages everything outside proof/ with `git add -A`. Refusing
# here first keeps this repo's rule: only files staged by name reach a commit.
if ! git diff --quiet -- . ':!proof' || [ -n "$(git ls-files --others --exclude-standard -- . ':!proof')" ]; then
  echo "Unstaged or untracked files exist outside proof/. Stage the files your change needs by name, and delete scratch files, so the proof matches the code it ran against." >&2
  exit 2
fi

export CARGO_BUILD_WARNINGS=deny
exec scripts/write-proof.sh \
  --ticket "$ticket" \
  --verification "scripts/verify.sh" \
  --fixture "the feature map in .claude/skills/verify/SKILL.md" \
  --shows "Every lane check's output and exit code, then each feature in the map run through the release binary, ending in RESULT: PASS." \
  -- \
  "scripts/check-pinned-scripts.sh" \
  "scripts/test-proof-tools.sh" \
  "cargo fmt --check" \
  "cargo clippy --all-targets --all-features --locked" \
  "cargo test --all-features --locked" \
  "cargo test --doc --locked" \
  "scripts/verify.sh"
