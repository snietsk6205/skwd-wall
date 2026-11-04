#!/usr/bin/env bash
# Adds "Sketch before transition" to skwd-wall: a Settings toggle under
# Motion -> Wallpaper transitions that draws the new wallpaper as a sketch,
# stroke by stroke, before whichever transition you picked.
#
# It builds patched copies of three skwd-wall programs (the app, the
# background service and the renderer) inside your home folder and points
# skwd-wall at them. Installed packages are never modified.
# Undo everything with: bash uninstall-sketch.sh
set -euo pipefail

here=$(cd -- "$(dirname -- "$0")" && pwd)
root="$HOME/.local/src/skwd-sketch"
opt="$HOME/.local/opt/skwd-sketch"
bin="$opt/bin"
unit_dir="$HOME/.config/systemd/user/skwd-walld.service.d"
dropin="$unit_dir/sketch-transition.conf"
apps="$HOME/.local/share/applications"
config="${XDG_CONFIG_HOME:-$HOME/.config}/skwd-wall-v2/config.json"
backup="$config.before-sketch"
marker="# Added by skwd sketch-transition installer"

say() { printf '\n==> %s\n' "$*"; }
fail() {
    printf '\nSTOPPED: %s\n' "$*" >&2
    printf 'Your skwd-wall setup was not changed by this step.\n' >&2
    exit 1
}

say "1/7 Checking what is needed"
for patch in skwd-paper skwd-deck skwd-wall; do
    [[ -f "$here/$patch.patch" ]] || fail "$patch.patch must be next to this script."
done
missing=()
for tool in git cargo rustup glslc pkg-config python3 systemctl; do
    command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
done
if ((${#missing[@]})); then
    fail "missing tools: ${missing[*]}
Install them with:
  sudo pacman -S --needed git rustup shaderc pkgconf python clang vulkan-headers vulkan-icd-loader ffmpeg dav1d libyuv alsa-lib libpulse libxkbcommon wayland fontconfig
  rustup default stable"
fi
[[ -f /usr/include/vulkan/vulkan.h ]] || fail "Vulkan headers missing. Run: sudo pacman -S --needed vulkan-headers"
for program in skwd-wall-v2 skwd-walld skwd-wall-vk; do
    command -v "$program" >/dev/null 2>&1 || fail "$program not found. Install skwd-wall v2 first."
done
system_bin=$(dirname -- "$(readlink -f -- "$(command -v skwd-walld)")")
version=$("$system_bin/skwd-wall-vk" --version 2>/dev/null | awk '{print $2}')
[[ -n "$version" ]] || version=$(skwd-wall-vk --version | awk '{print $2}')
[[ -n "$version" ]] || fail "could not read the installed skwd-wall version."
echo "Installed skwd-wall version: $version (programs in $system_bin)"

say "2/7 Downloading the matching source code (v$version)"
rm -rf "$root" "$HOME/.local/src/skwd-paper-sketch"
mkdir -p "$root"
for repo in skwd-paper skwd-deck skwd-wall skwd-lens; do
    git -c advice.detachedHead=false clone --quiet --depth 1 --branch "v$version" \
        "https://github.com/liixini/$repo.git" "$root/$repo" \
        || fail "no v$version source for $repo. This patch was made for 1.0.0-beta.26."
done

say "3/7 Adding the sketch feature"
for repo in skwd-paper skwd-deck skwd-wall; do
    git -C "$root/$repo" apply --check "$here/$repo.patch" \
        || fail "the $repo patch does not fit v$version (it was made for 1.0.0-beta.26)."
done
for repo in skwd-paper skwd-deck skwd-wall; do
    git -C "$root/$repo" apply "$here/$repo.patch"
done

say "4/7 Building (the first time takes 15-40 minutes)"
build() {
    local dir=$1
    shift
    (cd "$dir" && rustup toolchain install >/dev/null 2>&1 || true)
    (cd "$dir" && cargo build --release --locked "$@") \
        || fail "building $(basename "$dir") failed. Copy the last ~30 lines above and ask for help."
}
build "$root/skwd-paper" -p skwd-wall-vk
build "$root/skwd-deck" -p skwd-walld
build "$root/skwd-wall" --package skwd-wall --bin skwd-wall

say "5/7 Installing the patched programs into $opt"
rm -rf "$opt"
mkdir -p "$bin"
cp "$root/skwd-paper/target/release/skwd-wall-vk" "$bin/skwd-wall-vk"
cp "$root/skwd-deck/target/release/skwd-walld" "$bin/skwd-walld"
cp "$root/skwd-wall/target/release/skwd-wall" "$bin/skwd-wall-v2"
for program in "$system_bin"/skwd-*; do
    name=$(basename -- "$program")
    [[ -e "$bin/$name" ]] || ln -s "$program" "$bin/$name"
done
ln -s "$(dirname -- "$system_bin")/share" "$opt/share"
"$bin/skwd-wall-vk" --version >/dev/null || fail "the new renderer does not start."

say "6/7 Pointing skwd-wall at the patched programs"
exec_line=$(systemctl --user cat skwd-walld.service 2>/dev/null | grep -m1 '^ExecStart=.\+' || true)
[[ -n "$exec_line" ]] || fail "could not read the skwd-walld service. Is it enabled? (systemctl --user enable --now skwd-walld.service)"
command_part=${exec_line#ExecStart=}
first=${command_part%% *}
rest=""
[[ "$command_part" == *" "* ]] && rest=" ${command_part#* }"
[[ "$(basename -- "$first")" == "skwd-walld" ]] || fail "unexpected service command: $command_part"
mkdir -p "$unit_dir"
cat > "$dropin" <<EOF
$marker. Delete this file (or run uninstall-sketch.sh) to undo.
[Service]
ExecStart=
ExecStart=$bin/skwd-walld$rest
Environment=SKWD_WALL_PAPER_VK=$bin/skwd-wall-vk
Environment=SKWD_PAPER_VK_BIN=$bin/skwd-wall-vk
EOF
mkdir -p "$apps"
for desktop in /usr/share/applications/*skwd-wall*.desktop; do
    [[ -f "$desktop" ]] || continue
    target="$apps/$(basename -- "$desktop")"
    if [[ -f "$target" ]] && ! grep -q "$marker" "$target"; then
        echo "Leaving your own $target alone."
        continue
    fi
    { echo "$marker"; sed "s#^Exec=\(.*/\)\?skwd-wall-v2#Exec=$bin/skwd-wall-v2#" "$desktop"; } > "$target"
done

say "7/7 Turning the sketch on"
pkill -x skwd-wall-v2 2>/dev/null || true
pkill -x skwd-wall 2>/dev/null || true
if [[ -f "$config" ]]; then
    [[ -f "$backup" ]] || cp "$config" "$backup"
    python3 - "$config" "$backup" <<'PY'
import json, sys
path, backup = sys.argv[1], sys.argv[2]
with open(path) as f:
    cfg = json.load(f)
t = cfg.setdefault("transition", {})
if t.get("shader") == "sketch-draw":
    previous = "inkwell-drop"
    try:
        with open(backup) as f:
            previous = json.load(f).get("transition", {}).get("shader") or previous
    except OSError:
        pass
    t["shader"] = previous if previous != "sketch-draw" else "inkwell-drop"
t["enabled"] = True
t["sketchEnabled"] = True
t.setdefault("sketchStyle", "pencil")
t.setdefault("sketchDurationMs", 6000)
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
PY
fi
systemctl --user daemon-reload
systemctl --user restart skwd-walld.service
sleep 1
if ! systemctl --user is-active --quiet skwd-walld.service; then
    printf '\nThe background service did not restart. Run: bash %s/uninstall-sketch.sh\n' "$here" >&2
    exit 1
fi

say "Done."
echo "Open skwd-wall from your app launcher (or run: $bin/skwd-wall-v2)."
echo "If you open it with a keyboard shortcut, change that shortcut to run:"
echo "    $bin/skwd-wall-v2"
echo "The settings are in Settings -> Motion -> Wallpaper transitions -> Sketch before transition."
echo "To undo everything: bash $here/uninstall-sketch.sh"
