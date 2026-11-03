# CADTBT360_V1.012 — Geometry-source integrity and Windows build gate

Source branch: release/CADTBT360_V1.012
Parent: release/CADTBT360_V1.011
State: CODE COMMITTED. Windows CI, EXE and installer have not been verified.

## Implemented in code
- Detect repeated CAD area geometry by canonicalizing hexadecimal entity handles (AA, aa, 0AA are the same entity).
- Prevent one CAD region from representing both excavation and embankment in one cross-section even if handles have different string representations.
- Prevent two consecutive stations using one CAD area region to generate a duplicated AEA quantity.
- Add Rust regression test for both cases and ensure the new local test does not shadow its helper function.
- Increment visible desktop version, road QTO report title and default Excel filename to CADTBT360_V1.012.
- Windows workflow checks test discovery first, then runs the area-geometry source regression test and retains the existing AEA, marker linking, cargo check, release build and installer acceptance steps.

## Acceptance gates still open
- Neither the test run nor Windows cargo build has been observed PASS in this turn. Writing an Actions workflow does not certify EXE.
- The portable EXE/ZIP and Setup artifacts are expectations pending actual GitHub Actions execution and upload.
- Windows application startup, installation/uninstallation and road DWG/DXF benchmark drawings are not yet verified.
- Automatic multi-layout station detection and cut/fill transition-point interpolation remain unfinished.
- Source Cargo workspace directory remains CADTBT360_V1.000 for path compatibility; released app label is V1.012.

## Expected files after successful CI
CADTBT360_V1.012.exe
CADTBT360_V1.012_Portable.zip
CADTBT360_V1.012_Portable_SHA256.txt
CADTBT360_V1.012_Setup.exe
CADTBT360_V1.012_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1012-windows.yml
