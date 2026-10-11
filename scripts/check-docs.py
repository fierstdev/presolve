#!/usr/bin/env python3
"""Check local Markdown links in the public Presolve documentation."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parent.parent
LINK_PATTERN = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
SCANNED = [ROOT / "README.md", *sorted((ROOT / "docs").rglob("*.md"))]


def local_target(raw: str) -> str | None:
    target = raw.strip()
    if not target:
        return None

    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1].strip()

    # Markdown permits an optional title after a whitespace separator. None of
    # Presolve's docs need whitespace-bearing local paths, so split there.
    target = target.split(maxsplit=1)[0]

    lowered = target.lower()
    if (
        target.startswith("#")
        or target.startswith("/")
        or lowered.startswith("http://")
        or lowered.startswith("https://")
        or lowered.startswith("mailto:")
    ):
        return None

    target = unquote(target.split("#", 1)[0].split("?", 1)[0])
    return target or None


def main() -> int:
    failures: list[str] = []

    for source in SCANNED:
        if not source.exists():
            failures.append(f"missing documentation source: {source.relative_to(ROOT)}")
            continue

        text = source.read_text()

        for match in LINK_PATTERN.finditer(text):
            target = local_target(match.group(1))
            if target is None:
                continue

            resolved = (source.parent / target).resolve()

            try:
                resolved.relative_to(ROOT.resolve())
            except ValueError:
                failures.append(
                    f"{source.relative_to(ROOT)}: link escapes repository: {target}"
                )
                continue

            if not resolved.exists():
                failures.append(
                    f"{source.relative_to(ROOT)}: missing local target: {target}"
                )

    if failures:
        print("Documentation link check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print(f"Documentation link check passed ({len(SCANNED)} Markdown files).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
