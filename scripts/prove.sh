#!/usr/bin/env bash
set -euo pipefail

# Runs every lane check and the real-artifact verification, then writes the
# proof bundle (ADR 0005 in Etdev2/sweet-potato). Stage your change first; then
# commit proof/ alongside it.
#
# usage: scripts/prove.sh <owner/repo#issue>
ticket="${1:?usage: scripts/prove.sh <owner/repo#issue>}"
cd "$(dirname "$0")/.."

if ! git diff --quiet -- . ':!proof' || [ -n "$(git ls-files --others --exclude-standard -- . ':!proof')" ]; then
  echo "Stage every change outside proof/ first, so the proof matches the code it ran against." >&2
  exit 2
fi

rm -rf proof
mkdir -p proof
export CARGO_BUILD_WARNINGS=deny
commands='[]'

run_check() {
  local code=0
  echo "\$ $*" >> proof/checks.txt
  "$@" >> proof/checks.txt 2>&1 || code=$?
  echo "[exit $code]" >> proof/checks.txt
  commands=$(jq -c --arg c "$*" --argjson e "$code" '. + [{command: $c, exitCode: $e}]' <<< "$commands")
}

run_check cargo fmt --check
run_check cargo clippy --all-targets --all-features --locked
run_check cargo test --all-features --locked
run_check cargo test --doc --locked

verify_code=0
scripts/verify.sh > proof/verify.txt 2>&1 || verify_code=$?
case "$verify_code" in
  0) result=PASS ;;
  1) result=FAIL ;;
  *) result=INCONCLUSIVE ;;
esac
if [ "$result" = PASS ] && jq -e 'any(.exitCode != 0)' <<< "$commands" > /dev/null; then
  result=FAIL
fi

jq -n \
  --arg ticket "$ticket" \
  --arg digest "$(scripts/proof-digest.sh .)" \
  --arg at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg result "$result" \
  --argjson commands "$commands" '{
    schemaVersion: 1,
    ticket: $ticket,
    sourceDigest: $digest,
    capturedAt: $at,
    result: $result,
    verification: {command: "scripts/verify.sh", fixture: "the feature map in .claude/skills/verify/SKILL.md"},
    commands: $commands,
    behaviorChanged: true,
    evidence: [
      {path: "proof/verify.txt", kind: "terminal", shows: "Each feature in the map run through the release binary, with its output and exit code."},
      {path: "proof/checks.txt", kind: "log", shows: "Output of every lane check."}
    ]
  }' > proof/manifest.json

echo "Proof bundle written: $result. Commit proof/ with your change."
[ "$result" = PASS ]
