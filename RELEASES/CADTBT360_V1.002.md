# CADTBT360_V1.002 — Road earthwork transition update

Version: CADTBT360_V1.002
Parent release branch: release/CADTBT360_V1.001
Status: SOURCE COMMITTED; WINDOWS TESTS AND EXE NOT YET VERIFIED

## Implemented in Rust
- Separate explicit zero-area flags for cut and fill at each surveyed cross-section.
- Never silently equate missing geometry with zero area.
- Reject contradictory zero flag plus a positive CAD measurement.
- Compute AEA intervals with either endpoint explicitly zero; skip intervals where both endpoints are confirmed zero.
- Retain JSON backward compatibility: missing zero flags in legacy V1.001/V1.000 project files default to false.
- Introduce UI checkboxes for surveyed zero areas and distinguish verified zero from unknown in station table.
- Keep zero-area transition intervals visible in Excel preview counts and document their evidence as '0 (đã xác nhận)'.
- Add strict station-label parser for explicit Km text and regression tests; UI text/attribute enumeration is not wired up yet.
- Bump desktop title, workbook filename, and workflow artifact names to V1.002.

## Not completed
- Automatic canvas station recognition and text or block attribute extraction.
- Geometric cut/fill zero-crossing location interpolation between survey stations.
- Full Windows-native compilation/test pass and installation/runtime acceptance.
- Print Preview comparison against a real road takeoff project.
- Legacy Cargo workspace folder path retains CADTBT360_V1.000 as a technical source folder; it is not the V1.002 release name.

## Release gates
1. GitHub Actions V1.002 road tests pass on Windows.
2. Windows EXE compiles successfully.
3. Portable and Inno Setup artifacts are uploaded and installer tested.
4. Validate cut->zero, zero->fill, missing-end rejection and workbook report against real CAD drawing.

Workflow: .github/workflows/cadtbt360-v1002-windows.yml
