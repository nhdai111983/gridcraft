# shellcheck shell=bash
# Shared setup for the packaging scripts. Source it: `. "$(dirname "$0")/../env.sh"`.
#
# Exports:
#   ROOT                    workspace root
#   VERSION                 [workspace.package] version from Cargo.toml (override: GRIDCRAFT_VERSION)
#   DIST                    output directory for release artifacts (default: $ROOT/dist/release)
#   GRIDCRAFT_BUILD_SHA    git commit baked into the binaries (see crates/engine/src/build_info.rs)
#   GRIDCRAFT_BUILD_DATE   UTC build date, YYYY-MM-DD
#   CARGO_TARGET_DIR        cargo's target dir (default: $ROOT/target)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ROOT

# The version lives in exactly one place: `[workspace.package] version` in the root Cargo.toml.
# (`cargo xtask version` prints the same thing; awk avoids compiling xtask here.)
workspace_version() {
  awk '
    /^\[/ { in_pkg = ($0 == "[workspace.package]") ; next }
    in_pkg && $1 == "version" { gsub(/[" ]/, "", $3); print $3; exit }
  ' "$ROOT/Cargo.toml"
}

VERSION="${GRIDCRAFT_VERSION:-$(workspace_version)}"
if [ -z "$VERSION" ]; then
  echo "error: could not read [workspace.package] version from $ROOT/Cargo.toml" >&2
  exit 1
fi
export VERSION

DIST="${DIST:-$ROOT/dist/release}"
mkdir -p "$DIST"
export DIST

if [ -z "${GRIDCRAFT_BUILD_SHA:-}" ]; then
  GRIDCRAFT_BUILD_SHA="$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || true)"
fi
export GRIDCRAFT_BUILD_SHA
export GRIDCRAFT_BUILD_DATE="${GRIDCRAFT_BUILD_DATE:-$(date -u +%Y-%m-%d)}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

# Emit a GitHub Actions warning (plain stderr outside Actions).
warn() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then echo "::warning::$*"; else echo "warning: $*" >&2; fi
}

# Copy licence and readme files that exist into a package directory.
copy_docs() {
  local dest="$1" f
  for f in README.md LICENSE COPYRIGHT; do
    if [ -f "$ROOT/$f" ]; then cp "$ROOT/$f" "$dest/" || return 1; fi
  done
  copy_font_licences "$dest" || return 1
  copy_notices "$dest" || return 1
}

# NexusOffices: the licences, NOTICE (Apache-2.0 §4(d)) and ATTRIBUTION.md (the bundled fonts and
# assets with their licences) go in every package; the branded README links to NOTICE and
# ATTRIBUTION.md. So do the OFL texts of the repo's own embedded fonts (assets/fonts/OFL-*.txt).
# Run after copy_font_licences: a name that clashes with a craft-fonts OFL-<family>.txt on a
# case-insensitive filesystem then fails instead of overwriting it. Every copy is checked, because
# callers use `copy_notices … || …`, which turns errexit off inside the function.
copy_notices() {
  local dest="$1" f
  for f in LICENSE-MIT LICENSE-APACHE NOTICE ATTRIBUTION.md; do
    if [ ! -f "$ROOT/$f" ]; then echo "error: $ROOT/$f is missing" >&2; return 1; fi
    cp "$ROOT/$f" "$dest/" || return 1
  done
  for f in "$ROOT"/assets/fonts/OFL-*.txt; do
    [ -f "$f" ] || continue
    if [ -e "$dest/${f##*/}" ]; then echo "error: $dest/${f##*/} already exists (font licence name clash)" >&2; return 1; fi
    cp "$f" "$dest/" || return 1
  done
}

# Builds made with craft-fonts (CRAFT_FONTS_DIR, set by the release workflow) embed its fonts, so
# the package carries their licences as OFL-<family-dir>.txt. Nothing without CRAFT_FONTS_DIR.
copy_font_licences() {
  local dest="$1" f n=0
  if [ -n "${CRAFT_FONTS_DIR:-}" ]; then
    for f in "$CRAFT_FONTS_DIR"/fonts/*/OFL.txt; do
      [ -f "$f" ] || continue
      cp "$f" "$dest/OFL-$(basename "$(dirname "$f")").txt" || return 1
      n=$((n + 1))
    done
  fi
  # NexusOffices: a release build (CRAFT_FONTS_REQUIRED=1) embeds craft-fonts, so its licences must ship.
  if [ "${CRAFT_FONTS_REQUIRED:-}" = 1 ] && [ "$n" -eq 0 ]; then
    echo "error: CRAFT_FONTS_REQUIRED=1 but no fonts/*/OFL.txt in CRAFT_FONTS_DIR (${CRAFT_FONTS_DIR:-unset})" >&2
    return 1
  fi
}

# Portable SHA-256 of a file (prints just the hash).
sha256() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}
