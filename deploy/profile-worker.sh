#!/usr/bin/env sh
set -eu

# `wrangler check startup` cannot yet discover nested static-asset directories
# when it performs its own build. Feed it the successful dry-run bundle instead.
bundle_file="/tmp/janata-worker-bundle.mjs"
profile_file="/tmp/janata-worker-startup.cpuprofile"

wrangler deploy \
  --dry-run \
  --config deploy/wrangler.jsonc \
  --outfile "$bundle_file"

wrangler check startup \
  --workerBundle "$bundle_file" \
  --outfile "$profile_file"
