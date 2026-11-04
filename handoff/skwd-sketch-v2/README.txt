skwd-wall "Sketch before transition" (unofficial add-on, made for skwd-wall 1.0.0-beta.26)

Install:  bash install-sketch.sh
Undo:     bash uninstall-sketch.sh   (safe to run more than once)

After installing: Settings -> Motion -> Wallpaper transitions -> "Sketch before transition"
  - Draw a sketch first: on / off
  - Drawing style: Pencil on paper, Digital screen, Chalkboard, Blueprint,
    Ink brush, Charcoal, Notebook pen
  - Sketch duration (ms)
Your normal transition setting still works: the sketch is drawn first, then
whichever transition you picked runs from the finished sketch.

The installer replaces the older sketch-draw add-on automatically.
Run uninstall-sketch.sh BEFORE updating skwd-wall with yay/pacman.

Files it creates (all removed by uninstall-sketch.sh, except the settings backup):
  ~/.local/src/skwd-sketch/                 source code + build
  ~/.local/opt/skwd-sketch/                 patched programs
  ~/.config/systemd/user/skwd-walld.service.d/sketch-transition.conf
  ~/.local/share/applications/skwd-wall*.desktop   (launcher pointing to the patched app)
  ~/.config/skwd-wall-v2/config.json.before-sketch (settings backup)
