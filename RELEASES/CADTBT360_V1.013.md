# CADTBT360_V1.013 — Road cross-section evidence reuse guard

Source: release/CADTBT360_V1.013
Parent: release/CADTBT360_V1.012
Status: Code committed; Windows tests, EXE and Setup not yet verified.

## Implemented
- Canonicalize CAD area-source handles and reject use of the same geometric region in non-adjacent cross-sections, not just in consecutive stations.
- Detect area-source reuse when a region represents excavation at one station and fill at another.
- Add two regression tests: rejects_non_adjacent_area_reuse_across_three_stations, rejects_cross_kind_area_reuse_between_different_stations.
- Preserve explicit confirmed-zero versus missing-area semantics and Average End Area (AEA) computation.
- Update desktop version and Excel takeoff filename to V1.013.
- Add Windows test steps for both new cases before release build and portable/Setup packaging.

## Release acceptance still missing
- No successful Windows Cargo build or test run confirmed in this session.
- No EXE/ZIP/Setup artifacts confirmed uploaded; workflow files only describe intended steps.
- GUI startup, DWG/DXF road examples, Excel print and actual Windows installer require verification.
- Cross-layout station discovery and cut/fill switch interpolation remain unfinished.
- Source workspace folder CADTBT360_V1.000 is retained for Cargo compatibility.

## Planned files upon successful CI
CADTBT360_V1.013.exe
CADTBT360_V1.013_Portable.zip
CADTBT360_V1.013_Portable_SHA256.txt
CADTBT360_V1.013_Setup.exe
CADTBT360_V1.013_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1013-windows.yml
