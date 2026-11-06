#!/usr/bin/env python3
"""Verify that the distributable CAD 360 full-source ZIP is complete and clean.

No third-party dependencies. Run after 'git archive --format=zip HEAD'. Ensures
the released source archive is the exact committed source, including user guides.
"""
from __future__ import annotations

import subprocess
import sys
import zipfile
import hashlib
from pathlib import Path

REQUIRED = (
    "CADTBT360_V1.000/Cargo.toml",
    "CADTBT360_V1.000/Cargo.lock",
    "CADTBT360_V1.000/apps/cadcraft/src/main.rs",
    "CADTBT360_V1.000/apps/cadcraft/src/ai_raster_panel.rs",
    "CADTBT360_V1.000/crates/mcp/src/raster_vector.rs",
    "installer/CADTBT360_V1.025_Windows.iss",
    ".github/workflows/cad360-v1025-release.yml",
    "scripts/verify_mcp_executable.ps1",
    "scripts/validate_mcp_smoke_fixture.py",
    "scripts/verify_source_bundle.py",
    "docs/user-guide/build_word_guide.py",
    "docs/user-guide/CADTBT360_V1.025_HUONG_DAN_AI_TU_ANH_THANH_CAD.md",
    "docs/user-guide/CADTBT360_V1.025_GIAO_TRINH_TONG_HOP_TAT_CA_TINH_NANG.md",
    "scripts/validate_full_user_guide.py",
    "scripts/validate_word_guide.py",
    "docs/user-guide/CADTBT360_V1.025_GIAO_TRINH_TONG_HOP_TAT_CA_TINH_NANG.md",
    "scripts/validate_full_user_guide.py",
)

def committed_files() -> dict[str, str]:
    """List every regular file and symlink in the exact HEAD tree, not the worktree."""
    entries = subprocess.check_output(["git", "ls-tree", "-rz", "--full-tree", "HEAD"]).split(b"\0")
    result: dict[str, str] = {}
    for entry in entries:
        if not entry:
            continue
        header, path = entry.split(b"\t", 1)
        mode, kind, object_id = header.decode("ascii").split()
        if kind != "blob":
            raise ValueError(f"Unexpected git object in source tree: {path!r}")
        result[path.decode("utf-8", errors="surrogateescape")] = object_id
    return result


def build(archive: Path) -> None:
    """Write exact immutable Git blobs to ZIP without reading working-tree files."""
    files = committed_files()
    archive.parent.mkdir(parents=True, exist_ok=True)
    proc = subprocess.Popen(["git", "cat-file", "--batch"], stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        assert proc.stdin is not None and proc.stdout is not None
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=6, allowZip64=True) as z:
            for name, oid in sorted(files.items()):
                proc.stdin.write((oid + "\n").encode("ascii"))
                proc.stdin.flush()
                header = proc.stdout.readline().decode("ascii").strip().split()
                if len(header) != 3 or header[0] != oid or header[1] != "blob":
                    raise ValueError(f"Git blob header does not match requested object: {name}")
                remaining = int(header[2])
                with z.open(name, "w") as item:
                    while remaining:
                        chunk = proc.stdout.read(min(remaining, 65536))
                        if not chunk:
                            raise ValueError(f"Truncated source blob: {name}")
                        item.write(chunk)
                        remaining -= len(chunk)
                if proc.stdout.read(1) != b"\n":
                    raise ValueError(f"Missing cat-file blob separator for {name}")
        proc.stdin.close()
        if proc.wait(timeout=30) != 0:
            raise ValueError("git cat-file --batch failed")
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()


def verify(archive: Path) -> None:
    if not archive.is_file() or archive.stat().st_size < 4096:
        raise ValueError("Full source ZIP missing or suspiciously small")
    sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    if len(sha) != 40:
        raise ValueError("Cannot identify source commit")
    with zipfile.ZipFile(archive) as z:
        if z.testzip() is not None:
            raise ValueError("Corrupt source ZIP")
        names = set(z.namelist())
        files = committed_files()
        missing = set(files) - names
        extra = names - set(files)
        if missing or extra:
            raise ValueError(f"Source ZIP content differs from HEAD: missing={sorted(missing)[:10]}, extra={sorted(extra)[:10]}")
        required_missing = set(REQUIRED) - names
        if required_missing:
            raise ValueError(f"Full source ZIP missing required files: {sorted(required_missing)}")
        # Compare exact Git SHA-1 blob IDs, including header, for EVERY committed source file.
        for name, object_id in files.items():
            archived = z.read(name)
            payload = b"blob " + str(len(archived)).encode("ascii") + b"\0" + archived
            computed = hashlib.sha1(payload).hexdigest()
            if computed != object_id:
                raise ValueError(f"Source ZIP does not match commit {sha}: {name} (blob={computed}, expected={object_id})")
        if any(name.endswith((".exe", ".dll", ".pdb")) or "/target/" in name for name in names):
            raise ValueError("Full source ZIP unexpectedly contains build outputs")
        print(f"PASS: full CAD 360 source ZIP from commit {sha}, {len(names)} entries, required guides, all checked bytes match")

if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--build":
        build(Path(sys.argv[2]))
        verify(Path(sys.argv[2]))
    elif len(sys.argv) == 2:
        verify(Path(sys.argv[1]))
    else:
        raise SystemExit("Usage: python scripts/verify_source_bundle.py [--build] dist/v1025/CADTBT360_V1.025_Full_Source.zip")
