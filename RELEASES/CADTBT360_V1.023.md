# CADTBT360_V1.023 — Parallel development: Model/Paper Layout scan, cut/fill transition preview, DXF regression, Windows Setup

Branch: release/CADTBT360_V1.023
Parent: release/CADTBT360_V1.022
STATUS: source commits only, not a verified Windows release.

## Stream 1 — Multi-Layout plan station scanning (implemented source, pending CI)
- Added a CAD UI action "QUÉT CỌC MODEL + TẤT CẢ LAYOUT" which checks document model plus each Layout's paper-space entities.
- Scan bounded by a preflight 100,000-entity limit to avoid silently incomplete scanning; reports number of Layouts, matched stations and ambiguities.
- Resolve station marker handles from model or Layout paper space during source validation before Excel export.
- Detect handle reused in more than one space and reject ambiguous export.
- Added tests for markers across two paper Layouts, a model-only baseline and duplicate handle collision.

## Stream 2 — Cut/fill transition estimates (implemented source, not approved quantity)
- Added geometry-independent estimate_cut_fill_transition for the special case of surveyed opposite cut/fill end areas with explicit zero of the opposite type.
- Calculates a theoretical crossover station and triangular estimated cut/fill volumes assuming linear signed area variation.
- Renders conspicuous advisory estimates below the AEA road takeoff preview.
- THESE ESTIMATES ARE NOT EXPORTED TO EXCEL and do not overwrite measured AEA volumes. Actual measured transition cross-section is required for contractual quantity approval.
- Added regression tests for estimated transition location/volumes and preservation of ordinary AEA export.

## Stream 3 — DWG/DXF benchmark (partial: generated DXF regression added)
- Added generated-DXF import/export roundtrip Rust test using one model-space and one paper-space station marker.
- CI explicitly runs DXF roundtrip, multi-Layout scan and collision tests.
- NOT a real construction drawing DWG/DXF acceptance. No user-supplied construction DWG/DXF or engineer-approved benchmark/reference quantity set was available in this turn.
- Actual drawing-based quantity and Excel print/precision comparison remain open; not misreported as completed.

## Stream 4 — Windows packaging (source committed, not run-proven)
- Updated desktop version and Excel output identity to V1.023.
- Added checked-in CADTBT360_V1.023.iss installer script; Windows workflow references the checked-in installer definition.
- CI gates on tests for all three feature tracks before cargo check/release build, PE/ZIP validation, Portable artifact upload, Inno Setup and install/uninstall smoke.
- No confirmed GitHub Actions PASS or downloadable exe/setup artifact was observed.

## Acceptance checklist
- [x] Branch/source commits and tracked Windows installer script.
- [x] Static feature regression tests added to Cargo test suite.
- [ ] Rust regression tests PASS on Windows runner.
- [ ] Real DWG and DXF construction files evaluated against known-good manual quantities.
- [ ] Multi-layout station selection tested in actual Windows desktop application.
- [ ] Signed cut/fill transition assumptions independently verified with surveyor.
- [ ] Portable EXE runs in Windows desktop.
- [ ] Setup EXE successfully installs/uninstalls and is downloadable.

Workflow: .github/workflows/cadtbt360-v1023-windows.yml
