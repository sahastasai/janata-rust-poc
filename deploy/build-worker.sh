#!/usr/bin/env sh
set -eu

expected_version="0.8.5"
installed_version=""

if command -v worker-build >/dev/null 2>&1; then
  installed_version="$(worker-build --version)"
fi

if [ "$installed_version" != "$expected_version" ]; then
  cargo install --locked worker-build --version 0.8.5
fi

# The workspace release profile already uses panic=abort. The hidden recovery
# pipeline in worker-build 0.8.5 currently requires an externref table that the
# pinned stable toolchain does not emit for this crate, so use its legacy
# bundling path until that upstream mismatch is resolved.
worker-build --release --no-panic-recovery
