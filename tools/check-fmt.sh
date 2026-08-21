#!/usr/bin/env bash
#
# Format every Rust file in the repository.
#
#   tools/check-fmt.sh            # check only; non-zero exit if anything differs
#   tools/check-fmt.sh --write    # rewrite the files
#
# This exists because `cargo fmt --all` does not do the job here, for two
# separate reasons:
#
#  1. It only reaches workspace *members*. Every lesson's `solution/` crate
#     declares its own `[workspace]` table (see docs/conventions.md), so it is
#     not a member of the root workspace and `cargo fmt --all` has never
#     formatted a single one of them.
#
#  2. `cargo-fmt` passes every target path to rustfmt in one command line. With
#     roughly a hundred packages that list runs past 34 KB, over Windows'
#     32,767-byte limit, and the command dies with "The filename or extension
#     is too long. (os error 206)" before rustfmt runs at all.
#
# `git ls-files` gives exactly the tracked sources — no `target/`, no scratch
# files — and batching keeps every command line well inside the limit.

set -uo pipefail

cd "$(git rev-parse --show-toplevel)" || exit 2

check=1
if [ "${1:-}" = "--write" ]; then
  check=0
fi

edition="$(sed -n 's/^edition = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
edition="${edition:-2021}"

run_batch() {
  if [ "$check" = 1 ]; then
    rustfmt --edition "$edition" --check "$@"
  else
    rustfmt --edition "$edition" "$@"
  fi
}

status=0
count=0
batch=()

while IFS= read -r file; do
  batch+=("$file")
  count=$((count + 1))
  # 60 paths per call: comfortably inside the limit even for long lesson
  # paths, and few enough invocations that the whole run takes seconds.
  if [ "${#batch[@]}" -ge 60 ]; then
    run_batch "${batch[@]}" || status=1
    batch=()
  fi
done < <(git ls-files '*.rs')

if [ "${#batch[@]}" -gt 0 ]; then
  run_batch "${batch[@]}" || status=1
fi

if [ "$status" = 0 ]; then
  echo "formatting OK — $count files"
else
  echo
  echo "formatting differs — run 'tools/check-fmt.sh --write' to fix"
fi
exit "$status"
