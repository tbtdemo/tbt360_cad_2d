# CADTBT360_V1.020 — Five-version packaging checkpoint

Parent branch: release/CADTBT360_V1.019
Current branch: release/CADTBT360_V1.020
Status: SOURCE COMMITTED. Windows CI PASS and actual EXE/ZIP/Setup artifact NOT VERIFIED.

## Source changes
- Fix output station_label on kilometre rounding boundaries. A surveyed chainage at 999.999 m now rounds to Km1+000.00 instead of displaying invalid Km0+1000.00, and 1999.999 m becomes Km2+000.00.
- Add regression checks for kilometre formatting boundaries in the existing road station test.
- Increment desktop version label, road takeoff heading and default Excel output filename to CADTBT360_V1.020.
- Deduplicate repeated Windows CI test invocations: retain the complete takeoff_road::tests suite and road_station_matching_tests suite and a specific Km rounding regression discovery gate. Avoid repeatedly executing the same tests through a dozen independent steps.
- Generate Inno Setup script as UTF-8 with BOM to preserve Vietnamese installation text.
- Add an always-running GitHub Actions run summary showing commit, status, and whether each expected file actually exists on the runner.
- Preserve release build, portable-first EXE and ZIP artifact upload before installer, PE verification, SHA256 checks, setup installation/uninstallation verification and full setup artifact publishing.

## Required validation, not yet observed
- Windows Rust cargo check + release build passes.
- Road test suites PASS on real Windows runner.
- Downloadable CADTBT360_V1.020.exe and Portable.zip artifacts exist.
- Setup.exe compiles and Windows runner installation and uninstall PASS.
- Real DWG/DXF drawing, QTO Excel layout and Windows desktop startup are verified.

## Still open in product
- Multi-layout automatic station scanning.
- Interpolating genuine measured cut/fill transition sections.
- Independent road drawing benchmark and printed Excel formatting acceptance.

## Expected outputs after a successful workflow
- CADTBT360_V1.020.exe
- CADTBT360_V1.020_Portable.zip
- CADTBT360_V1.020_Portable_SHA256.txt
- CADTBT360_V1.020_Setup.exe
- CADTBT360_V1.020_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1020-windows.yml
