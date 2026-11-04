#!/usr/bin/env bash
# Undoes custom/install.sh: skwd-wall goes back to the official programs.
# The official packages were never changed. Safe to run more than once.
#
# Options:  --keep-source   keep ~/.local/src/skwd-custom (faster re-install later)
#
# Your settings (~/.config/skwd-wall-v2/config.json) are kept. The extra
# settings of the custom build are simply ignored by the official app.
set -euo pipefail

main() {
    local keep_source=0
    for arg in "$@"; do
        case "$arg" in
            --keep-source) keep_source=1 ;;
            -h|--help) sed -n '2,9p' "$0"; return 0 ;;
            *) printf 'unknown option: %s\n' "$arg" >&2; return 1 ;;
        esac
    done
    [[ $EUID -ne 0 ]] || { echo "Run this as your normal user, not with sudo." >&2; return 1; }

    local src=${SKWD_CUSTOM_SRC:-$HOME/.local/src/skwd-custom}
    local unit_dir=${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/skwd-walld.service.d
    local apps=${XDG_DATA_HOME:-$HOME/.local/share}/applications

    pkill -x skwd-wall-v2 2>/dev/null || true
    pkill -x skwd-wall 2>/dev/null || true

    rm -f "$unit_dir/skwd-custom.conf" "$unit_dir/sketch-transition.conf"
    rmdir "$unit_dir" 2>/dev/null || true
    echo "Removed the service override."

    local desktop
    for desktop in "$apps"/*skwd-wall*.desktop; do
        [[ -f "$desktop" ]] || continue
        if grep -q -e "# Added by the skwd-custom installer" \
            -e "# Added by skwd sketch-transition installer" "$desktop"; then
            rm -f "$desktop"
            echo "Removed launcher override $(basename -- "$desktop")."
        fi
    done

    systemctl --user daemon-reload
    if systemctl --user restart skwd-walld.service; then
        echo "Background service restarted with the official programs."
    else
        echo "Could not restart skwd-walld; run: systemctl --user restart skwd-walld.service" >&2
    fi

    rm -rf "$HOME/.local/opt/skwd-custom" "$HOME/.local/opt/skwd-custom.new" \
        "$HOME/.local/opt/skwd-custom.previous" "$HOME/.local/opt/skwd-sketch"
    echo "Deleted the custom programs."
    if ((keep_source)); then
        echo "Kept the source code in $src."
    else
        # The scripts may live inside $src; bash has already read this whole function.
        rm -rf "$src" "$HOME/.local/src/skwd-sketch"
        echo "Deleted the source code and build files."
    fi

    cat <<'EOF'

All done. skwd-wall now runs the official programs again.
If you changed Hyprland shortcuts to ~/.local/opt/skwd-custom/..., change them back:
    bind = SUPER, N, exec, skwd-wall-v2
    bind = SUPER, W, exec, skwd-helm random
then run:  hyprctl reload
EOF
}

main "$@"
exit 0
