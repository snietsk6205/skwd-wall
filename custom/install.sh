#!/usr/bin/env bash
# skwd-wall custom build — install it, or update it to the newest version.
#
# What it does, in short:
#   1. checks the tools it needs (and prints the one pacman command to get them),
#   2. downloads/updates the source of the custom build into ~/.local/src/skwd-custom,
#   3. builds every skwd-wall program from that source (first time: 15-40 minutes),
#   4. puts the programs in ~/.local/opt/skwd-custom (nothing outside your home folder),
#   5. points the skwd-wall background service and app launcher at them.
#
# The official skwd-wall packages are never changed. Undo everything with:
#   bash ~/.local/src/skwd-custom/skwd-wall/custom/uninstall.sh
#
# Options:  --no-restart   build and install, but leave the running service alone
#           --keep-going   continue even if the official skwd-wall version differs
set -euo pipefail

SELF_HASH=$(sha256sum < "${BASH_SOURCE[0]}")

main() {
    local owner=${SKWD_CUSTOM_OWNER:-snietsk6205}
    local branch=${SKWD_CUSTOM_BRANCH:-my-custom}
    local git_base=${SKWD_CUSTOM_GIT_BASE:-https://github.com/$owner}
    local src=${SKWD_CUSTOM_SRC:-$HOME/.local/src/skwd-custom}
    local opt=$HOME/.local/opt/skwd-custom
    local unit_dir=${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/skwd-walld.service.d
    local dropin=$unit_dir/skwd-custom.conf
    local apps=${XDG_DATA_HOME:-$HOME/.local/share}/applications
    local marker="# Added by the skwd-custom installer"
    local restart=1 keep_going=0
    for arg in "$@"; do
        case "$arg" in
            --no-restart) restart=0 ;;
            --keep-going) keep_going=1 ;;
            -h|--help) sed -n '2,17p' "$0"; return 0 ;;
            *) fail "unknown option: $arg" ;;
        esac
    done

    [[ $EUID -ne 0 ]] || fail "run this as your normal user, not with sudo or as root."

    say "1/7 Checking what is needed"
    check_tools
    local system_walld
    system_walld=$(official_program skwd-walld) \
        || fail "the official skwd-wall is not installed. Install it first:
    yay -S skwd-wall-v2-bin skwd-lens-bin
    systemctl --user daemon-reload
    systemctl --user enable --now skwd-walld.service"
    local system_bin system_prefix
    system_bin=$(dirname -- "$system_walld")
    system_prefix=$(dirname -- "$system_bin")
    [[ -d "$system_prefix/share/skwd-wall-v2" ]] \
        || fail "could not find skwd-wall's data folder ($system_prefix/share/skwd-wall-v2)."
    local unit_file
    unit_file=$(systemctl --user show -P FragmentPath skwd-walld.service 2>/dev/null || true)
    [[ -n "$unit_file" && -f "$unit_file" ]] \
        || fail "the skwd-walld background service is missing. Run:
    systemctl --user daemon-reload
    systemctl --user enable --now skwd-walld.service"
    echo "Official skwd-wall found in $system_bin"

    say "2/7 Getting the custom source code (branch $branch)"
    mkdir -p "$src"
    local repo
    for repo in skwd-wall skwd-deck skwd-paper; do
        sync_repo "$git_base/$repo" "$branch" "$src/$repo"
    done
    maybe_restart_with_new_script "$src/skwd-wall/custom/install.sh" "$@"
    local version
    version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$src/skwd-paper/Cargo.toml" | head -n1)
    [[ -n "$version" ]] || fail "could not read the version of the custom source."
    sync_tag "https://github.com/liixini/skwd-lens.git" "v$version" "$src/skwd-lens"
    echo "Custom build is based on skwd-wall $version"
    local installed
    installed=$("$system_bin/skwd-wall-vk" --version 2>/dev/null | awk '{print $2}' || true)
    if [[ -n "$installed" && "$installed" != "$version" ]]; then
        printf '\nNOTE: your official skwd-wall is %s, the custom build is based on %s.\n' "$installed" "$version"
        printf 'The custom build replaces all skwd-wall programs, so this usually works,\n'
        printf 'but it is safest when both match (ask for the forks to be updated).\n'
        if ((!keep_going)); then
            fail "versions differ. Re-run with --keep-going to install anyway."
        fi
    fi

    say "3/7 Building (first time 15-40 minutes, updates are much faster)"
    local toolchain
    toolchain=$(sed -n 's/^channel = "\(.*\)"/\1/p' "$src/skwd-paper/rust-toolchain.toml" | head -n1)
    if [[ -n "$toolchain" ]]; then
        rustup toolchain install "$toolchain" --profile minimal >/dev/null 2>&1 \
            || fail "could not install Rust $toolchain with rustup (internet needed)."
    fi
    export CARGO_TARGET_DIR=$src/target
    build "$src/skwd-paper" -p skwd-wall-vk -p skwd-wall-still -p skwd-paper -p paper-tinier
    build "$src/skwd-deck" -p skwd-walld -p skwd-helm -p skwd-wall-scan -p skwd-wall-effects -p skwd-steam
    build "$src/skwd-wall" --package skwd-wall --bin skwd-wall
    build "$src/skwd-lens" -p skwd-lens

    say "4/7 Installing the programs into $opt"
    local out=$CARGO_TARGET_DIR/release stage=$opt.new
    rm -rf "$stage"
    mkdir -p "$stage/bin" "$stage/lib/skwd-paper"
    local name
    for name in skwd-wall-vk skwd-wall-still skwd-walld skwd-helm skwd-wall-scan skwd-wall-effects \
        skwd-steam skwd-lens; do
        install -m 755 "$out/$name" "$stage/bin/$name"
    done
    install -m 755 "$out/skwd-wall" "$stage/bin/skwd-wall-v2"
    install -m 755 "$out/skwd-paper" "$stage/bin/skwd-paper-v2"
    install -m 755 "$out/skwd-paper-tinier" "$stage/lib/skwd-paper/skwd-paper-tinier"
    ln -s "$system_prefix/share" "$stage/share"
    # Steam's own library cannot be built from source; borrow it from the package.
    local steam_lib
    for steam_lib in "$system_bin/libsteam_api.so" "$system_prefix/lib/skwd-deck/libsteam_api.so"; do
        if [[ -e "$steam_lib" ]]; then
            ln -s "$steam_lib" "$stage/bin/libsteam_api.so"
            break
        fi
    done
    "$stage/bin/skwd-wall-vk" --version >/dev/null 2>&1 || fail "the new renderer does not start."
    local previous=""
    if [[ -d "$opt" ]]; then
        previous=$opt.previous
        rm -rf "$previous"
        mv "$opt" "$previous"
    fi
    mv "$stage" "$opt"

    say "5/7 Pointing the background service at the custom programs"
    local exec_line command_part first rest=""
    exec_line=$(grep -m1 '^ExecStart=.\+' "$unit_file" || true)
    [[ -n "$exec_line" ]] || fail "could not read the service command from $unit_file."
    command_part=${exec_line#ExecStart=}
    first=${command_part%% *}
    [[ "$command_part" == *" "* ]] && rest=" ${command_part#* }"
    [[ "$(basename -- "$first")" == "skwd-walld" ]] || fail "unexpected service command: $command_part"
    mkdir -p "$unit_dir"
    rm -f "$unit_dir/sketch-transition.conf"   # from the older sketch-only installer
    cat > "$dropin" <<EOF
$marker. Delete this file (or run custom/uninstall.sh) to undo.
[Service]
ExecStart=
ExecStart=$opt/bin/skwd-walld$rest
Environment=SKWD_WALL_PAPER_VK=$opt/bin/skwd-wall-vk
Environment=SKWD_WALL_PAPER_STILL=$opt/bin/skwd-wall-still
EOF

    say "6/7 Pointing the app launcher at the custom app"
    mkdir -p "$apps"
    local desktop target
    for desktop in "$apps"/*skwd-wall*.desktop; do
        [[ -f "$desktop" ]] || continue
        if grep -q -e "$marker" -e "# Added by skwd sketch-transition installer" "$desktop"; then
            rm -f "$desktop"
        fi
    done
    for desktop in "$system_prefix"/share/applications/*skwd-wall*.desktop; do
        [[ -f "$desktop" ]] || continue
        target=$apps/$(basename -- "$desktop")
        if [[ -f "$target" ]]; then
            echo "Leaving your own $target alone."
            continue
        fi
        { echo "$marker"; sed -e "s#^Exec=\([^ ]*/\)\?skwd-wall-v2#Exec=$opt/bin/skwd-wall-v2#" \
            -e "s#^TryExec=.*#TryExec=$opt/bin/skwd-wall-v2#" "$desktop"; } > "$target"
    done
    rm -rf "$HOME/.local/opt/skwd-sketch"   # older sketch-only install, replaced by this one

    say "7/7 Restarting skwd-wall with the custom programs"
    pkill -x skwd-wall-v2 2>/dev/null || true
    pkill -x skwd-wall 2>/dev/null || true
    systemctl --user daemon-reload
    if ((restart)); then
        systemctl --user restart skwd-walld.service || true
        sleep 2
        if ! systemctl --user is-active --quiet skwd-walld.service; then
            printf '\nThe background service did not start with the custom programs. Going back.\n' >&2
            if [[ -n "$previous" && -d "$previous" ]]; then
                rm -rf "$opt"
                mv "$previous" "$opt"
            else
                rm -f "$dropin"
                rm -rf "$opt"
                grep -l -e "$marker" "$apps"/*skwd-wall*.desktop 2>/dev/null | xargs -r rm -f
            fi
            systemctl --user daemon-reload
            systemctl --user restart skwd-walld.service || true
            fail "see the log with:  journalctl --user -u skwd-walld -n 50"
        fi
    fi
    [[ -n "$previous" ]] && rm -rf "$previous"
    if ! systemctl --user is-enabled --quiet skwd-walld.service 2>/dev/null; then
        printf '\nNOTE: the background service does not start at login yet. To fix that, run:\n'
        printf '    systemctl --user enable skwd-walld.service\n'
    fi

    say "Done. The custom skwd-wall is installed."
    cat <<EOF

Open it from your app launcher, or run:
    $opt/bin/skwd-wall-v2

Hyprland keyboard shortcuts must use the full path. In ~/.config/hypr/hyprland.conf:
    bind = SUPER, N, exec, $opt/bin/skwd-wall-v2
    bind = SUPER, W, exec, $opt/bin/skwd-helm random
then run:  hyprctl reload

New settings: Settings -> Motion -> "Sketch before transition".
Update later:  bash $src/skwd-wall/custom/install.sh
Undo all:      bash $src/skwd-wall/custom/uninstall.sh
EOF
}

say() { printf '\n==> %s\n' "$*"; }

fail() {
    printf '\nSTOPPED: %s\n' "$*" >&2
    printf 'Your running skwd-wall was not changed by the step that failed.\n' >&2
    exit 1
}

check_tools() {
    local missing=() tool
    local -A package=(
        [git]=git [rustup]=rustup [glslc]=shaderc [pkg-config]=pkgconf [clang]=clang
        [nasm]=nasm [cc]=base-devel [make]=base-devel [python3]=python
    )
    for tool in git rustup glslc pkg-config clang nasm cc make python3; do
        command -v "$tool" >/dev/null 2>&1 || missing+=("${package[$tool]}")
    done
    command -v systemctl >/dev/null 2>&1 || fail "systemd is required."
    [[ -f /usr/include/vulkan/vulkan.h ]] || missing+=(vulkan-headers)
    local lib
    local -A libpkg=(
        [libavcodec]=ffmpeg [dav1d]=dav1d [alsa]=alsa-lib [libpulse]=libpulse
        [xkbcommon]=libxkbcommon [wayland-client]=wayland [fontconfig]=fontconfig [vulkan]=vulkan-icd-loader
    )
    if command -v pkg-config >/dev/null 2>&1; then
        for lib in libavcodec dav1d alsa libpulse xkbcommon wayland-client fontconfig vulkan; do
            pkg-config --exists "$lib" 2>/dev/null || missing+=("${libpkg[$lib]}")
        done
    fi
    [[ -f /usr/include/libyuv.h ]] || missing+=(libyuv)
    if ((${#missing[@]})); then
        local unique
        unique=$(printf '%s\n' "${missing[@]}" | sort -u | tr '\n' ' ')
        fail "some build tools are missing. Install them with this command, then run this script again:
    sudo pacman -S --needed $unique"
    fi
    command -v cargo >/dev/null 2>&1 \
        || fail "cargo is missing. Install Rust with:  sudo pacman -S --needed rustup"
}

# Prints the official program's real path, skipping this custom build's copy.
official_program() {
    local name=$1 path
    while IFS= read -r path; do
        path=$(readlink -f -- "$path")
        [[ "$path" == "$HOME/.local/opt/"* ]] && continue
        printf '%s\n' "$path"
        return 0
    done < <(type -ap "$name")
    return 1
}

sync_repo() {
    local url=$1 branch=$2 dir=$3
    if [[ -d "$dir/.git" ]]; then
        git -C "$dir" fetch --quiet origin "$branch" || fail "could not download updates for $(basename "$dir") (internet?)."
        git -C "$dir" checkout --quiet -B "$branch" "origin/$branch"
        git -C "$dir" reset --quiet --hard "origin/$branch"
    else
        rm -rf "$dir"
        git clone --quiet --branch "$branch" "$url" "$dir" || fail "could not download $url (internet?)."
    fi
    echo "$(basename "$dir"): $(git -C "$dir" log -1 --format='%h %s')"
}

sync_tag() {
    local url=$1 tag=$2 dir=$3
    if [[ -d "$dir/.git" ]] && [[ "$(git -C "$dir" describe --tags --exact-match 2>/dev/null)" == "$tag" ]]; then
        echo "$(basename "$dir"): $tag (already downloaded)"
        return
    fi
    rm -rf "$dir"
    git -c advice.detachedHead=false clone --quiet --depth 1 --branch "$tag" "$url" "$dir" \
        || fail "could not download $url at $tag."
    echo "$(basename "$dir"): $tag"
}

# After an update, run the new copy of this script instead of the old one.
maybe_restart_with_new_script() {
    local new=$1
    shift
    [[ -n "${SKWD_CUSTOM_RESTARTED:-}" || ! -f "$new" ]] && return 0
    if [[ "$(sha256sum < "$new")" != "$SELF_HASH" ]]; then
        echo "The installer itself was updated; continuing with the new version."
        SKWD_CUSTOM_RESTARTED=1 exec bash "$new" "$@"
    fi
}

build() {
    local dir=$1
    shift
    (cd "$dir" && cargo build --release --locked "$@") \
        || fail "building $(basename "$dir") failed. Copy the last ~40 lines above and ask for help."
}

main "$@"
exit 0
