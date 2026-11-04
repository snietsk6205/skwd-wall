#!/usr/bin/env bash
# Undoes install-sketch.sh: skwd-wall goes back to its own programs, and the
# sketch settings are removed. The installed skwd-wall packages were never
# modified. Safe to run more than once.
set -euo pipefail

root="$HOME/.local/src/skwd-sketch"
opt="$HOME/.local/opt/skwd-sketch"
unit_dir="$HOME/.config/systemd/user/skwd-walld.service.d"
dropin="$unit_dir/sketch-transition.conf"
apps="$HOME/.local/share/applications"
config="${XDG_CONFIG_HOME:-$HOME/.config}/skwd-wall-v2/config.json"
backup="$config.before-sketch"
marker="# Added by skwd sketch-transition installer"

pkill -x skwd-wall-v2 2>/dev/null || true
pkill -x skwd-wall 2>/dev/null || true

rm -f "$dropin"
rmdir "$unit_dir" 2>/dev/null || true
echo "Removed the service override."

for desktop in "$apps"/*skwd-wall*.desktop; do
    [[ -f "$desktop" ]] || continue
    if grep -q "$marker" "$desktop"; then
        rm -f "$desktop"
        echo "Removed launcher override $(basename -- "$desktop")."
    fi
done

if [[ -f "$config" ]] && command -v python3 >/dev/null 2>&1; then
    python3 - "$config" "$backup" <<'PY'
import json, os, sys
path, backup = sys.argv[1], sys.argv[2]
with open(path) as f:
    cfg = json.load(f)
t = cfg.setdefault("transition", {})
for key in ("sketchEnabled", "sketchStyle", "sketchDurationMs"):
    t.pop(key, None)
if t.get("shader") == "sketch-draw":
    previous = "inkwell-drop"
    if os.path.exists(backup):
        with open(backup) as f:
            previous = json.load(f).get("transition", {}).get("shader") or previous
    t["shader"] = previous if previous != "sketch-draw" else "inkwell-drop"
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
print("Removed the sketch settings.")
PY
fi

systemctl --user daemon-reload
systemctl --user restart skwd-walld.service
echo "Background service restarted with the official programs."

rm -rf "$opt" "$root" "$HOME/.local/src/skwd-paper-sketch"
echo "Deleted the patched programs and their source code."
echo "All done. If you changed a keyboard shortcut to the patched app, change it back to: skwd-wall-v2"
