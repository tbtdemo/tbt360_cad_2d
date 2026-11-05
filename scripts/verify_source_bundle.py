#!/usr/bin/env python3
"""Verify that the distributable CAD 360 full-source ZIP is complete and clean.

No third-party dependencies. Run after 'git archive --format=zip HEAD'. Ensures
the released source archive is the exact committed source, including user guides.
"""
from __future__ import annotations

import subprocess
import sys
import zipfile
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
)

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
        missing = set(REQUIRED) - names
        if missing:
            raise ValueError(f"Full source ZIP missing files: {sorted(missing)}")
        for name in REQUIRED:
            committed = subprocess.check_output(["git", "show", f"HEAD:{name}"])
            archived = z.read(name)
            if archived != committed:
                raise ValueError(f"Source ZIP does not match commit {sha}: {name}")
        if any(name.endswith((".exe", ".dll", ".pdb")) or "/target/" in name for name in names):
            raise ValueError("Full source ZIP unexpectedly contains build outputs")
        print(f"PASS: full CAD 360 source ZIP from commit {sha}, {len(names)} entries, required guides, all checked bytes match")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python scripts/verify_source_bundle.py dist/v1025/CADTBT360_V1.025_Full_Source.zip")
    verify(Path(sys.argv[1]))
