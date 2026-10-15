#!/usr/bin/env python3
"""Check local links in public Presolve Markdown and static-site documentation."""

from __future__ import annotations

import re
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parent.parent
SITE_ROOT = ROOT / "site"
LINK_PATTERN = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
SCANNED = [ROOT / "README.md", *sorted((ROOT / "docs").rglob("*.md"))]
SITE_SCANNED = sorted(SITE_ROOT.rglob("*.html"))


class SiteLinkParser(HTMLParser):
    """Collect local-link candidates from static HTML attributes."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.targets: list[str] = []

    def handle_starttag(
        self, tag: str, attrs: list[tuple[str, str | None]]
    ) -> None:
        del tag
        for name, value in attrs:
            if name in {"href", "src"} and value:
                self.targets.append(value)


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


def site_target(source: Path, raw: str) -> Path | None:
    target = raw.strip()
    if not target or target.startswith("#") or target.startswith("//"):
        return None

    lowered = target.lower()
    if lowered.startswith(
        ("http://", "https://", "mailto:", "data:", "javascript:")
    ):
        return None

    target = unquote(target.split("#", 1)[0].split("?", 1)[0])
    if not target:
        return None

    if target.startswith("/"):
        resolved = SITE_ROOT / target.lstrip("/")
    else:
        resolved = source.parent / target

    if target.endswith("/") or resolved.is_dir():
        resolved /= "index.html"

    return resolved.resolve()


def check_markdown_links(failures: list[str]) -> None:
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


def check_site_links(failures: list[str]) -> None:
    for source in SITE_SCANNED:
        parser = SiteLinkParser()
        parser.feed(source.read_text())

        for raw in parser.targets:
            resolved = site_target(source, raw)
            if resolved is None:
                continue

            try:
                resolved.relative_to(SITE_ROOT.resolve())
            except ValueError:
                failures.append(
                    f"{source.relative_to(ROOT)}: site link escapes site root: {raw}"
                )
                continue

            if not resolved.exists():
                failures.append(
                    f"{source.relative_to(ROOT)}: missing site target: {raw}"
                )


def main() -> int:
    failures: list[str] = []

    check_markdown_links(failures)
    check_site_links(failures)

    if failures:
        print("Documentation link check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print(
        "Documentation link check passed "
        f"({len(SCANNED)} Markdown files, {len(SITE_SCANNED)} site pages)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
