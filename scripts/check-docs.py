#!/usr/bin/env python3
"""Check public Presolve Markdown and static-site documentation."""

from __future__ import annotations

import re
import sys
import xml.etree.ElementTree as ET
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parent.parent
SITE_ROOT = ROOT / "site"
LINK_PATTERN = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
SCANNED = [ROOT / "README.md", *sorted((ROOT / "docs").rglob("*.md"))]
SITE_SCANNED = sorted(SITE_ROOT.rglob("*.html"))
PUBLIC_SITE_SCANNED = [
    path for path in SITE_SCANNED if SITE_ROOT / "assets" not in path.parents
]


class SiteLinkParser(HTMLParser):
    """Collect local-link candidates and anchor IDs from static HTML."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.targets: list[str] = []
        self.hrefs: list[str] = []
        self.ids: set[str] = set()

    def handle_starttag(
        self, tag: str, attrs: list[tuple[str, str | None]]
    ) -> None:
        attributes = dict(attrs)
        element_id = attributes.get("id")
        if element_id:
            self.ids.add(element_id)

        if tag == "a" and attributes.get("name"):
            self.ids.add(attributes["name"] or "")

        for name, value in attrs:
            if name in {"href", "src"} and value:
                self.targets.append(value)
            if name == "href" and value:
                self.hrefs.append(value)


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


def is_external_site_target(raw: str) -> bool:
    target = raw.strip().lower()
    return target.startswith(
        ("http://", "https://", "mailto:", "data:", "javascript:", "//")
    )


def site_target(source: Path, raw: str) -> Path | None:
    target = raw.strip()
    if not target or target.startswith("#") or is_external_site_target(target):
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


def parse_site_page(path: Path, cache: dict[Path, SiteLinkParser]) -> SiteLinkParser:
    resolved = path.resolve()
    cached = cache.get(resolved)
    if cached is not None:
        return cached

    parser = SiteLinkParser()
    parser.feed(resolved.read_text())
    cache[resolved] = parser
    return parser


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


def check_site_links(
    failures: list[str], cache: dict[Path, SiteLinkParser]
) -> None:
    for source in SITE_SCANNED:
        parser = parse_site_page(source, cache)

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


def check_site_fragments(
    failures: list[str], cache: dict[Path, SiteLinkParser]
) -> None:
    for source in PUBLIC_SITE_SCANNED:
        parser = parse_site_page(source, cache)

        for raw in parser.hrefs:
            target = raw.strip()
            if not target or is_external_site_target(target):
                continue

            split = urlsplit(target)
            if not split.fragment:
                continue

            fragment = unquote(split.fragment)
            if split.path:
                resolved = site_target(source, target)
                if resolved is None or not resolved.exists():
                    # Missing files are reported by check_site_links.
                    continue
            else:
                resolved = source.resolve()

            target_parser = parse_site_page(resolved, cache)
            if fragment not in target_parser.ids:
                failures.append(
                    f"{source.relative_to(ROOT)}: missing anchor #{fragment} "
                    f"in {resolved.relative_to(ROOT)}"
                )


def check_public_site_shell(failures: list[str]) -> None:
    required_assets = ('href="/navigation.css"', 'src="/site.js"')

    for source in PUBLIC_SITE_SCANNED:
        text = source.read_text()
        relative = source.relative_to(ROOT)

        if 'name="viewport"' not in text:
            failures.append(f"{relative}: missing responsive viewport metadata")
        if 'class="site-header"' not in text:
            failures.append(f"{relative}: missing shared site header")

        for required in required_assets:
            if required not in text:
                failures.append(f"{relative}: missing shared navigation asset {required}")

        if SITE_ROOT / "docs" in source.parents:
            for required in (
                'href="/docs/docs.css"',
                'class="docs-layout shell"',
                'class="docs-nav"',
                'class="docs-content"',
            ):
                if required not in text:
                    failures.append(
                        f"{relative}: missing documentation shell marker {required}"
                    )


def public_site_url(source: Path) -> str:
    relative = source.relative_to(SITE_ROOT)
    if relative == Path("index.html"):
        return "https://presolve.dev/"

    path = relative.as_posix()
    if path.endswith("/index.html"):
        path = path[: -len("index.html")]
    return f"https://presolve.dev/{path}"


def check_sitemap(failures: list[str]) -> None:
    sitemap = SITE_ROOT / "sitemap.xml"
    if not sitemap.exists():
        failures.append("site/sitemap.xml: missing sitemap")
        return

    try:
        root = ET.fromstring(sitemap.read_text())
    except ET.ParseError as error:
        failures.append(f"site/sitemap.xml: invalid XML: {error}")
        return

    namespace = {"s": "http://www.sitemaps.org/schemas/sitemap/0.9"}
    listed = {
        element.text.strip()
        for element in root.findall("s:url/s:loc", namespace)
        if element.text and element.text.strip()
    }

    expected = {public_site_url(source) for source in PUBLIC_SITE_SCANNED}
    for missing in sorted(expected - listed):
        failures.append(f"site/sitemap.xml: missing public page {missing}")


def main() -> int:
    failures: list[str] = []
    cache: dict[Path, SiteLinkParser] = {}

    check_markdown_links(failures)
    check_site_links(failures, cache)
    check_site_fragments(failures, cache)
    check_public_site_shell(failures)
    check_sitemap(failures)

    if failures:
        print("Documentation/site check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print(
        "Documentation/site check passed "
        f"({len(SCANNED)} Markdown files, {len(PUBLIC_SITE_SCANNED)} public site pages)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
