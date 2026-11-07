#!/usr/bin/env python3
"""Append Fluent keys to one .ftl file in every locale (custom build helper).

Usage: add_locale_keys.py <relative ftl path, e.g. settings/performance.ftl> <keys.json>

keys.json: {"key-name": {"en-US": "English", "fr-FR": "Français", ...}, ...}
Locales without a translation get the English text. Keys that already exist
in a file are replaced in place, so re-running is safe.
"""
import json
import pathlib
import re
import sys

root = pathlib.Path(__file__).resolve().parents[2] / "locales"
target, spec = sys.argv[1], json.loads(pathlib.Path(sys.argv[2]).read_text())
for locale in sorted(p.name for p in root.iterdir() if p.is_dir()):
    path = root / locale / target
    text = path.read_text() if path.exists() else ""
    for key, values in spec.items():
        value = values.get(locale, values["en-US"]).replace("\n", " ")
        line = f"{key} = {value}"
        pattern = re.compile(rf"^{re.escape(key)} = .*$", re.M)
        if pattern.search(text):
            text = pattern.sub(lambda _m: line, text)
        else:
            if text and not text.endswith("\n"):
                text += "\n"
            text += line + "\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    print(f"{locale}: {path.relative_to(root.parent)}")
