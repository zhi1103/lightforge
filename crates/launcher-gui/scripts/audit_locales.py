#!/usr/bin/env python3
"""Check Fluent key parity/references and reject new hard-coded GUI copy."""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
GUI = ROOT / "crates/launcher-gui"
I18N = GUI / "src/i18n.rs"


def messages(path: Path) -> tuple[dict[str, set[str]], dict[str, set[str]], list[str]]:
    found: dict[str, set[str]] = {}
    variables: dict[str, set[str]] = {}
    errors: list[str] = []
    current: str | None = None
    for line_no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line or line[0].isspace() or line.startswith("#"):
            if current and line.strip():
                variables[current].update(re.findall(r"\$([\w-]+)", line))
            continue
        match = re.match(r"^([\w-]+)(?:\.([\w-]+))?\s*=", line)
        if match:
            current = match.group(1)
            attribute = match.group(2) or "value"
            if attribute in found.setdefault(current, set()):
                errors.append(f"{path.name}:{line_no}: duplicate {current}.{attribute}")
            found[current].add(attribute)
            variables.setdefault(current, set()).update(re.findall(r"\$([\w-]+)", line))
        elif line.strip() and not line.startswith("#"):
            current = None
    return found, variables, errors


def main() -> int:
    en, en_vars, errors = messages(GUI / "locales/en-US.ftl")
    zh, zh_vars, zh_errors = messages(GUI / "locales/zh-CN.ftl")
    errors.extend(zh_errors)
    if en.keys() != zh.keys():
        errors.append(f"locale key mismatch: en-only={sorted(en.keys() - zh.keys())}, zh-only={sorted(zh.keys() - en.keys())}")
    for key in en.keys() & zh.keys():
        if en[key] != zh[key]:
            errors.append(f"attribute mismatch for {key}: en={sorted(en[key])}, zh={sorted(zh[key])}")
        if en_vars.get(key, set()) != zh_vars.get(key, set()):
            errors.append(f"variable mismatch for {key}: en={sorted(en_vars.get(key, set()))}, zh={sorted(zh_vars.get(key, set()))}")

    source = I18N.read_text(encoding="utf-8")
    keys = set(re.findall(r'pub\(crate\) const \w+: &str = "([\w-]+)";', source)) - {"en-US", "zh-CN"}
    # Product-title ids are selected from stable product-code matches rather
    # than passed as literals at each UI call site.
    keys.update(re.findall(r'"(product-title-[\w-]+)"', source))
    rust = "\n".join(p.read_text(encoding="utf-8") for p in (GUI / "src").rglob("*.rs"))
    keys.update(re.findall(r'i18n::text(?:_with_args)?\(\s*(?:&?self\.config\.interface_language|language),\s*"([\w-]+)"', rust))
    # Category labels and similar enums carry Fluent ids through a mapping
    # method rather than passing the id directly to `i18n::text`.
    keys.update(key for key in re.findall(r'"([\w-]+)"', rust) if key.startswith(("report-category-", "settings-doctor-", "maintenance-")))
    for key in sorted(keys - en.keys()):
        errors.append(f"Rust references missing Fluent message: {key}")

    # Check newly added GUI copy. Existing legacy copy is audited and migrated
    # directory-by-directory; this guard keeps new literal labels from joining it.
    try:
        diff = subprocess.run(
            ["git", "diff", "HEAD", "--unified=0", "--", "crates/launcher-gui/src"],
            cwd=ROOT, check=True, capture_output=True, text=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError) as error:
        errors.append(f"could not inspect source diff: {error}")
        diff = ""
    ui_call = re.compile(r'\b(?:text|muted|secondary|secondary_tall|action|action_button|group_label|tab|nav|setting|empty|section|text_input|placeholder)\s*\([^\n]*"([^"\\]{3,})"')
    for line in diff.splitlines():
        if not line.startswith("+") or line.startswith("+++"):
            continue
        added = line[1:]
        if "i18n::text" in added or "i18n::text_with_args" in added or added.lstrip().startswith("//"):
            continue
        match = ui_call.search(added)
        if match and re.search(r"[A-Za-z]{2}", match.group(1)):
            errors.append(f"new hard-coded GUI copy: {added.strip()}")

    # Source-copy guard for the header/settings and the explicitly migrated
    # GUI boundaries: navigation, backup/launch task results, setup progress,
    # report attachments, wizard pickers, and update state. This checks added
    # diff lines in those scopes for direct widget/status/error/result copy.
    # Provider values, paths, commands, protocol labels, logs, CLI/core output,
    # and arbitrary strings outside these scopes are intentionally out of
    # bounds. It is not a semantic scan of every string in the crate.
    scoped = [
        "crates/launcher-gui/src/view/header/",
        "crates/launcher-gui/src/view/settings/",
        "crates/launcher-gui/src/view/companions.rs",
        "crates/launcher-gui/src/update/",
        "crates/launcher-gui/src/app/launch.rs",
        "crates/launcher-gui/src/app/nav.rs",
        "crates/launcher-gui/src/app/queue.rs",
        "crates/launcher-gui/src/app/library.rs",
        "crates/launcher-gui/src/app/launch.rs",
        "crates/launcher-gui/src/tasks/transfer.rs",
        "crates/launcher-gui/src/tasks/upgrade.rs",
        "crates/launcher-gui/src/tasks/backups.rs",
        "crates/launcher-gui/src/tasks/launch.rs",
        "crates/launcher-gui/src/provision.rs",
        "crates/launcher-gui/src/report.rs",
        "crates/launcher-gui/src/view/dialogs/bug_report.rs",
        "crates/launcher-gui/src/wizard/update.rs",
    ]
    try:
        scoped_diff = subprocess.run(
            ["git", "diff", "HEAD", "--unified=0", "--", *scoped],
            cwd=ROOT, check=True, capture_output=True, text=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError) as error:
        errors.append(f"could not inspect scoped source diff: {error}")
        scoped_diff = ""
    state_copy = re.compile(
        r"\b(?:self\.)?(?:status|error|cache_notice|drive_notice|addon_auto_note|proton_notice|steam_shortcut|point_error)\s*=.*?[\"']([^\"']*[A-Za-z]{2}[^\"']*)[\"']"
    )
    result_copy = re.compile(r'\b(?:Ok|Err)\s*\(\s*(?:format!\s*\(\s*)?["\']([^"\']*[A-Za-z]{2}[^"\']*)["\']')
    view_copy = re.compile(
        r"\b(?:text|tooltip|secondary|action|empty|section|nav|tab|setting|placeholder)\s*\([^\n]*[\"']([^\"']*[A-Za-z]{2}[^\"']*)[\"']"
    )
    scoped_file = ""
    for line in scoped_diff.splitlines():
        if line.startswith("+++ b/"):
            scoped_file = line[6:]
            continue
        if not line.startswith("+") or line.startswith("+++"):
            continue
        added = line[1:]
        if "i18n::text" in added or "i18n::text_with_args" in added or added.lstrip().startswith("//"):
            continue
        match = state_copy.search(added) if any(name in scoped_file for name in ("/update/", "/app/")) else view_copy.search(added)
        if not match and ("/tasks/" in scoped_file or scoped_file.endswith("/provision.rs")):
            match = result_copy.search(added)
        if match:
            errors.append(f"new hard-coded GUI state/copy in {scoped_file}: {added.strip()}")

    if errors:
        print("Fluent coverage audit failed:", file=sys.stderr)
        print("\n".join(f"- {error}" for error in errors), file=sys.stderr)
        return 1
    print(f"Fluent coverage audit passed: {len(en)} messages, en-US/zh-CN keys and Rust references match.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
