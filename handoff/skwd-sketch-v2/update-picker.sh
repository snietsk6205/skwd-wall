#!/usr/bin/env bash
# Rebuilds only the skwd-wall app (the picker) with the updated skwd-wall.patch,
# so the "Sketch before transition" page shows up in Settings -> Motion.
# Needs install-sketch.sh to have been run before. Takes a few minutes.
set -euo pipefail

here=$(cd -- "$(dirname -- "$0")" && pwd)
src="$HOME/.local/src/skwd-sketch/skwd-wall"
bin="$HOME/.local/opt/skwd-sketch/bin"

fail() { printf '\nSTOPPED: %s\nNothing was changed.\n' "$*" >&2; exit 1; }

[[ -f "$here/skwd-wall.patch" ]] || fail "skwd-wall.patch must be next to this script."
[[ -d "$src/.git" && -x "$bin/skwd-wall-v2" ]] || fail "run install-sketch.sh first."

echo "==> Updating the app source"
git -C "$src" checkout --quiet -- .
git -C "$src" apply --check "$here/skwd-wall.patch" || fail "the new patch does not fit."
git -C "$src" apply "$here/skwd-wall.patch"

echo "==> Rebuilding the app (a few minutes)"
(cd "$src" && cargo build --release --locked --package skwd-wall --bin skwd-wall) \
    || fail "the build failed. Copy the last ~30 lines above and ask for help."

pkill -x skwd-wall-v2 2>/dev/null && sleep 1 || true
cp "$src/target/release/skwd-wall" "$bin/skwd-wall-v2"
echo "==> Done. Open skwd-wall again: Settings -> Motion -> Sketch before transition."
