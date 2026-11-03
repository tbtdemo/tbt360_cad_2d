# CADTBT360_V1.015 — Road takeoff surveyed range coverage

Branch: release/CADTBT360_V1.015
Parent: release/CADTBT360_V1.014
State: SOURCE COMMITTED. No confirmed Windows build PASS, installable EXE or Setup artifact.

## Implemented
- Added validate_range_station_coverage to require real cross-sections at both explicit endpoints of the requested Km range.
- A station range which starts or ends between surveyed cross-sections now emits an error instead of silently dropping its partial boundary volume.
- Added the same validation to the on-screen road AEA interval preview and Excel report generation.
- No automatic interpolation or fabricated geometry is performed: an engineer must supply actual surveyed boundary cross-section data.
- Added regression tests for exact boundaries, missing initial station, missing final station and an unrestricted range.
- Updated desktop version, road QTO interface and Excel workbook filename to V1.015.
- New Windows workflow adds the regression test as a release gate while retaining AEA tests, Cargo check/build, portable-first artifact upload, Inno Setup smoke test and SHA-256.

## Acceptance still open
- No Windows build/test PASS has been verified from a workflow execution.
- No versioned Windows EXE, Portable ZIP or Setup has been confirmed available for downloading.
- Real DWG/DXF road takeoff and Excel printed report acceptance tests remain open.
- Multi-layout scanning and transition between excavation/fill remain unimplemented.
- Source Cargo workspace retains directory CADTBT360_V1.000.

## Expected files once CI succeeds
CADTBT360_V1.015.exe
CADTBT360_V1.015_Portable.zip
CADTBT360_V1.015_Portable_SHA256.txt
CADTBT360_V1.015_Setup.exe
CADTBT360_V1.015_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1015-windows.yml
