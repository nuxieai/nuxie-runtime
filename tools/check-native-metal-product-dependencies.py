#!/usr/bin/env python3
"""Reject renderer implementations forbidden from the native Metal product graph."""

from __future__ import annotations

import re
import sys
from collections.abc import Iterable


FORBIDDEN_PACKAGE = re.compile(r"^(?:wgpu(?:-|$)|naga(?:-|$)|dawn(?:-|$))", re.IGNORECASE)
PACKAGE_LINE = re.compile(r"^([^\s]+)\s+v\d")


def forbidden_packages(lines: Iterable[str]) -> list[str]:
    found: set[str] = set()
    for raw_line in lines:
        line = raw_line.strip()
        match = PACKAGE_LINE.match(line)
        if match and FORBIDDEN_PACKAGE.match(match.group(1)):
            found.add(match.group(1))
    return sorted(found, key=str.casefold)


def main() -> int:
    forbidden = forbidden_packages(sys.stdin)
    if not forbidden:
        return 0
    print(
        "error: native Metal product cargo graph retains forbidden renderer packages: "
        + ", ".join(forbidden),
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
