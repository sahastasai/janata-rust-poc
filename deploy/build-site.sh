#!/usr/bin/env sh
set -eu

repo_root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ui_output="$repo_root/target/dx/janata-ui/release/web/public"
site_output="$repo_root/dist/site"

command -v dx >/dev/null 2>&1 || {
  echo "Dioxus CLI (dx) is required; install the pinned 0.7.10 release." >&2
  exit 1
}
command -v mdbook >/dev/null 2>&1 || {
  echo "mdBook is required to build the hosted contributor guide." >&2
  exit 1
}

# Dioxus content hashes change whenever the UI changes, but `dx build` does not
# remove superseded hashed files. Clear only its generated public directory so
# neither benchmarks nor deployments count or ship unreachable stale assets.
case "$ui_output" in
  "$repo_root"/target/dx/janata-ui/release/web/public) rm -rf -- "$ui_output" ;;
  *) echo "Refusing to clear unexpected Dioxus output: $ui_output" >&2; exit 1 ;;
esac

(cd "$repo_root/apps/janata-ui" && dx build --web --release)
(cd "$repo_root" && mdbook build docs/book)
(cd "$repo_root" && RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps)

# `dist` is ignored, generated POC output. Keep the deletion narrowly scoped
# to this one staging directory so repeated builds cannot retain stale assets.
mkdir -p "$site_output"
rsync -a --delete --exclude '*.br' --exclude '*.gz' "$ui_output/" "$site_output/"
mkdir -p "$site_output/docs"
rsync -a --delete "$repo_root/dist/docs/guide/" "$site_output/docs/guide/"
rsync -a --delete "$repo_root/target/doc/" "$site_output/docs/api/"
rsync -a --delete "$repo_root/benchmarks/evidence/" "$site_output/docs/evidence/"
cp "$repo_root/deploy/static/_headers" "$site_output/_headers"

echo "Assembled POC site at $site_output"
