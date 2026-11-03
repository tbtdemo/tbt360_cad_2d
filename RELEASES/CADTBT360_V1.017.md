# CADTBT360_V1.017 — Complete road-range approval before Excel release

Source: release/CADTBT360_V1.017
Parent: release/CADTBT360_V1.016
Status: Source committed; Windows CI PASS / actual EXE and Setup unverified.

## Actual changes
- Fix a partial export bug: prior code considered an explicitly selected station range publishable if at least one pair of adjacent cross-sections was confirmed; this silently omitted later unapproved intervals.
- The confirmed-only publication check now validates every adjacent cross-section pair in the requested station range, failing with the exact offending station start/end when either cross-section is pending.
- Unconfirmed draft preview/export remains permitted by explicit confirmed_only=false.
- Add Rust regression test selected_road_range_rejects_unapproved_gap_after_first_valid_pair, including an approved earlier pair plus pending final station.
- Desktop title, road QTO report and default Excel workbook filename bumped to CADTBT360_V1.017.
- New Windows workflow runs regression gate along with existing road tests, Cargo compilation, portable-first Windows EXE, ZIP/PE verification, Inno Setup installer and silent install/uninstall smoke tests.

## What has NOT been verified
- Actual GitHub Actions run success, Windows executable, setup artifact, and downloading of deliverables.
- Native GUI startup, actual CAD DWG/DXF cross-section validation and Excel print view.
- Multi-layout automatic station scanning and cut/fill transition geometry interpolation remain unfinished.
- Source Cargo directory retains legacy CADTBT360_V1.000 layout.

## Expected artifacts only after successful CI
CADTBT360_V1.017.exe
CADTBT360_V1.017_Portable.zip
CADTBT360_V1.017_Portable_SHA256.txt
CADTBT360_V1.017_Setup.exe
CADTBT360_V1.017_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1017-windows.yml
