#!/usr/bin/env bash
set -euo pipefail

# Fails a pull request whose proof bundle is missing, malformed, not PASS, or
# captured for different code than the pull request now contains.
repo="${1:-.}"
manifest="$repo/proof/manifest.json"
here="$(cd "$(dirname "$0")" && pwd)"

fail() {
  echo "::error file=proof/manifest.json::$1"
  exit 1
}

[ -f "$manifest" ] || fail "No proof bundle. Run the project's verification skill, then commit proof/manifest.json and its evidence."
jq empty "$manifest" 2>/dev/null || fail "proof/manifest.json is not valid JSON."

problems=$(jq -r '
  def nonempty: type == "string" and length > 0;
  def kinds: ["screenshot", "video", "trace", "log", "terminal"];

  (select(.schemaVersion != 1) | "schemaVersion must be 1."),
  (select(.ticket | nonempty and test("^[^/]+/[^/#]+#[0-9]+$") | not)
    | "ticket must look like owner/repo#123."),
  (select(.sourceDigest | nonempty and test("^sha256:[0-9a-f]{64}$") | not)
    | "sourceDigest must come from scripts/proof-digest.sh."),
  (select(.capturedAt | nonempty | not) | "capturedAt is missing."),
  (select(.result | IN("PASS", "FAIL", "INCONCLUSIVE") | not)
    | "result must be PASS, FAIL or INCONCLUSIVE."),
  (select(.result | IN("FAIL", "INCONCLUSIVE"))
    | "Verification reported \(.result). Only a PASS bundle can merge. Fix the change or the environment and re-run verification."),
  (select((.verification.command | nonempty) and (.verification.fixture | nonempty) | not)
    | "verification needs the command that was run and the fixture it used."),
  (select((.commands | type) != "array" or (.commands | length) == 0)
    | "commands must list every check run, with its exit code."),
  (.commands[]? | select((.command | nonempty | not) or (.exitCode | type) != "number")
    | "Each command needs command and exitCode."),
  (.commands[]? | select((.exitCode | type) == "number" and .exitCode != 0)
    | "\(.command) exited \(.exitCode). Proof needs every check to pass."),
  (select((.behaviorChanged | type) != "boolean") | "behaviorChanged must be true or false."),
  (select(.behaviorChanged == true and ((.evidence | type) != "array" or (.evidence | length) == 0))
    | "Behavior changed, so evidence from the real artifact is required."),
  (select(.behaviorChanged == false and (.noBehaviorReason | nonempty | not))
    | "behaviorChanged is false, so noBehaviorReason must say why no user-visible proof applies."),
  (.evidence[]? | select((.path | nonempty and startswith("proof/") and (contains("..") | not)) | not)
    | "Evidence path \(.path | tojson) must sit inside proof/."),
  (.evidence[]? | select(.kind | IN(kinds[]) | not)
    | "Evidence kind \(.kind | tojson) must be one of: \(kinds | join(", ")).")
  , (.evidence[]? | select(.shows | nonempty | not)
    | "Evidence \(.path) needs shows: what a reader should see in it.")
' "$manifest")

if [ -n "$problems" ]; then
  while IFS= read -r problem; do
    echo "::error file=proof/manifest.json::$problem"
  done <<< "$problems"
  exit 1
fi

while IFS= read -r path; do
  git -C "$repo" ls-files --error-unmatch -- "$path" >/dev/null 2>&1 \
    || fail "Evidence $path is not committed."
done < <(jq -r '.evidence[]?.path' "$manifest")

cap=$((5 * 1024 * 1024))
size=$(git -C "$repo" ls-files -z -- proof | (cd "$repo" && xargs -0 -r cat) | wc -c)
[ "$size" -le "$cap" ] \
  || fail "proof/ holds $size bytes, over the 5 MB cap. Compress screenshots, trim logs, or link a long video from the pull request instead."

claimed=$(jq -r '.sourceDigest' "$manifest")
actual=$("$here/proof-digest.sh" "$repo")
[ "$claimed" = "$actual" ] \
  || fail "Proof is stale. It was captured for $claimed, but the code is now $actual. Re-run verification on the current code and commit a new bundle."

echo "Proof bundle is valid: $(jq -r '.result' "$manifest") for $(jq -r '.ticket' "$manifest")."
