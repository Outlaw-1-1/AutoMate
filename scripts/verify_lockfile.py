#!/usr/bin/env python3
"""Validate Cargo.lock for obvious corruption patterns.

Checks:
1) Cargo.lock is parseable as TOML.
2) No duplicate [[package]] entries with the same (name, version).
"""

from __future__ import annotations

import sys
from collections import Counter
from pathlib import Path

try:
    import tomllib  # py311+
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib


def main() -> int:
    lock_path = Path("Cargo.lock")
    if not lock_path.exists():
        print("error: Cargo.lock not found", file=sys.stderr)
        return 2

    try:
        data = tomllib.loads(lock_path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        print(f"error: failed to parse Cargo.lock: {exc}", file=sys.stderr)
        return 1

    packages = data.get("package", [])
    counts = Counter((pkg.get("name"), pkg.get("version")) for pkg in packages)
    dupes = [(name, version, count) for (name, version), count in counts.items() if count > 1]

    if dupes:
        print("error: duplicate package entries found in Cargo.lock:", file=sys.stderr)
        for name, version, count in sorted(dupes):
            print(f"  - {name} {version} appears {count} times", file=sys.stderr)
        return 1

    print("ok: Cargo.lock parsed and has no duplicate name+version package entries")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
