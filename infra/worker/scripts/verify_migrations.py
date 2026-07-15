#!/usr/bin/env python3
"""Verify immutable migration contents and, after apply, the remote D1 ledger."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


WORKER_DIR = Path(__file__).resolve().parents[1]
MIGRATIONS_DIR = WORKER_DIR / "migrations"
CHECKSUMS_PATH = MIGRATIONS_DIR / "checksums.sha256"
MIGRATION_NAME = re.compile(r"^(\d{4})_[a-z0-9_]+\.sql$")


def fail(message: str) -> None:
    print(f"migration verification failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def verified_local_names() -> list[str]:
    files = sorted(MIGRATIONS_DIR.glob("*.sql"))
    names = [path.name for path in files]
    numbers = []
    for name in names:
        match = MIGRATION_NAME.fullmatch(name)
        if match is None:
            fail(f"invalid migration filename: {name}")
        numbers.append(int(match.group(1)))
    if numbers != list(range(1, len(numbers) + 1)):
        fail(f"migration numbers must be contiguous from 0001: {names}")

    expected: dict[str, str] = {}
    for line in CHECKSUMS_PATH.read_text().splitlines():
        if not line.strip():
            continue
        parts = line.split()
        if len(parts) != 2 or not re.fullmatch(r"[0-9a-f]{64}", parts[0]):
            fail(f"invalid checksum entry: {line}")
        relative_path = parts[1]
        if relative_path in expected:
            fail(f"duplicate checksum entry: {relative_path}")
        expected[relative_path] = parts[0]

    actual_paths = {f"migrations/{name}" for name in names}
    if set(expected) != actual_paths:
        missing = sorted(actual_paths - set(expected))
        extra = sorted(set(expected) - actual_paths)
        fail(f"checksum manifest mismatch; missing={missing}, extra={extra}")

    for path in files:
        relative_path = f"migrations/{path.name}"
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected[relative_path]:
            fail(
                f"{relative_path} changed after its checksum was recorded; "
                "restore it and add a new numbered migration"
            )
    return names


def remote_names(database: str) -> list[str]:
    result = subprocess.run(
        [
            "wrangler",
            "d1",
            "execute",
            database,
            "--remote",
            "--json",
            "--command",
            "SELECT name FROM d1_migrations ORDER BY id",
        ],
        cwd=WORKER_DIR,
        check=True,
        capture_output=True,
        text=True,
    )
    payload = json.loads(result.stdout)
    if not payload or not payload[0].get("success"):
        fail("remote D1 migration query was not successful")
    return [row["name"] for row in payload[0].get("results", [])]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--remote", action="store_true")
    parser.add_argument("--database", default="jayjay_stats")
    args = parser.parse_args()

    local = verified_local_names()
    if args.remote:
        remote = remote_names(args.database)
        if remote != local:
            fail(f"remote D1 ledger differs; local={local}, remote={remote}")
    suffix = " and remote D1 ledger" if args.remote else ""
    print(f"Verified {len(local)} migration checksums{suffix}.")


if __name__ == "__main__":
    main()
