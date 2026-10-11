#!/usr/bin/env bash
set -uo pipefail

# Runs the release binary the way an operator does and checks each feature in
# the feature map (.claude/skills/verify/SKILL.md). Prints a transcript, then
# exits 0 for PASS, 1 for FAIL and 2 for INCONCLUSIVE.
cd "$(dirname "$0")/.."

if ! cargo build --release --locked --quiet; then
  echo "RESULT: INCONCLUSIVE (the release binary did not build)"
  exit 2
fi
bin="${CARGO_TARGET_DIR:-target}/release/rust-pilot"
failed=0

# case <feature> <expected exit> <text the output must contain> <args...>
case_() {
  local feature="$1" want_code="$2" want_text="$3"
  shift 3
  local out code
  out=$("$bin" "$@" 2>&1)
  code=$?
  echo "\$ rust-pilot $*"
  echo "$out"
  echo "[exit $code]"
  if [ "$code" -eq "$want_code" ] && grep -qF -- "$want_text" <<< "$out"; then
    echo "ok   $feature"
  else
    echo "FAIL $feature: wanted exit $want_code and output containing \"$want_text\""
    failed=1
  fi
  echo
}

case_ "stage lookup: unverified" 0 "no builder runs" stage-0-unverified
case_ "stage lookup: supervised" 0 "Et inspects every step" stage-1-supervised
case_ "stage lookup: proven" 0 "Et still merges" stage-2-proven
case_ "unknown stage is rejected" 2 "use stage-0-unverified, stage-1-supervised or stage-2-proven" stage-3-automerge
case_ "missing argument prints usage" 2 "usage: rust-pilot <trust-stage>"

if [ "$failed" -eq 0 ]; then
  echo "RESULT: PASS"
  exit 0
fi
echo "RESULT: FAIL"
exit 1
