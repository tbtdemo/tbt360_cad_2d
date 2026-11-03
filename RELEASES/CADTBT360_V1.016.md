# CADTBT360_V1.016 — Approved station range export guard

Branch: release/CADTBT360_V1.016
Parent: release/CADTBT360_V1.015
Status: Source committed. Windows build, portable EXE and Setup artifact remain unverified.

## Changes committed
- Added validate_range_export_readiness for an explicitly selected road station interval.
- An explicitly selected range for confirmed-only quantity export requires at least one adjacent pair of confirmed cross-sections. Otherwise return an actionable Vietnamese error rather than silently omitting road earthworks from a combined Excel report.
- Keep draft exports permissive for unconfirmed sections, as previously intended.
- Reuse existing strict checks that the selected Km start and end are actual surveyed station boundaries; avoid inventing partial interval earthwork areas.
- Add Rust regression test for confirmed/draft modes and absent explicit road range.
- Increment desktop display label, road takeoff heading and default Excel name to CADTBT360_V1.016.
- Windows workflow V1.016 runs the new regression test and retains earlier AEA, road marker, area reuse tests, cargo checks, x64 release build, portable-before-installer upload, PE/ZIP checks, Inno Setup install/uninstall checks and SHA256 checksums.

## Remaining requirements
- There is no verified passing Windows CI run or downloadable EXE/Setup from this commit.
- Need native Windows GUI smoke test, real DWG/DXF QTO benchmark, and Excel print validation.
- Multi-layout station discovery and geometry-based cut/fill transition interpolation are pending.
- Cargo source directory retains historical CADTBT360_V1.000 name.

## Expected files only upon successful CI
CADTBT360_V1.016.exe
CADTBT360_V1.016_Portable.zip
CADTBT360_V1.016_Portable_SHA256.txt
CADTBT360_V1.016_Setup.exe
CADTBT360_V1.016_SHA256.txt

Windows workflow: .github/workflows/cadtbt360-v1016-windows.yml
