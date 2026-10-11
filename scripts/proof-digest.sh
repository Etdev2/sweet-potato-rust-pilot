#!/usr/bin/env bash
set -euo pipefail

# Prints the digest a proof bundle is tied to: every staged file outside proof/,
# by path, mode and content. Stage the change before running it.
repo="${1:-.}"

listing=$(git -C "$repo" ls-files --stage --full-name -- ':/' ':(top,exclude)proof')
printf 'sha256:%s\n' "$(printf '%s\n' "$listing" | sha256sum | cut -d' ' -f1)"
