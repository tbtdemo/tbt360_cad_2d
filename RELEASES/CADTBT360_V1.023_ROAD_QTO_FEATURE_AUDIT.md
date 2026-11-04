# CADTBT360 V1.023 — Road QTO development before Windows packaging

Base: release/CADTBT360_V1.023
Development branch: feature/CADTBT360_road-qto-parallel-v1023
Do not merge or publish as a release before the feature CI and Windows GUI acceptance pass.

## 1. Selective station scanning, multiple CAD Layouts
- Model Space is included in every full scan.
- The UI can optionally select the Paper Space Layouts to scan; no selected checkboxes means all layouts.
- The 100,000-entity preflight counts only Model Space plus the selected Layouts, not unrelated layouts.
- Matching is based on explicit unambiguous Km text, preserving source handles and not automatically approving measurements.
- If the same handle appears multiple times in the scanned Model/Paper spaces, BOTH candidates are rejected. Automatic matching must never quietly accept the first candidate of an ambiguous handle.
- Export checks a handle across Model and all Paper Layouts, rejecting any duplicates even if only one layout was scanned.

## 2. Cut/fill crossover
- This is an advisory LINEAR signed-area interpolation from a single cut-positive endpoint to a fill-positive endpoint (or the reverse).
- Both endpoints require valid geometry evidence, with the opposite area explicitly declared zero.
- It predicts crossover station and illustrative split triangular quantities, but these results are not measured survey sections and are NEVER substituted into contractual/exported AEA results.
- Tests cover both cut-to-fill and fill-to-cut, malformed CAD source evidence, and the unchanged measured AEA output.
- A separate physically surveyed transition section, if available, must be entered and confirmed before using its quantities in final works reports.

## 3. CAD import acceptance
- Synthetic DXF source roundtrip already exists in the automatic Rust tests.
- A new OPT-IN ignored test accepts a PAIR of real drawings in equivalent DWG and DXF formats, plus engineer-verified reference station labels. It parses both files and checks the expected Km labels and full station list equality.
- It does NOT verify area, volume or Excel print results. Those require independent reference sheets and real measurements.
- Both input files must represent the same drawing revision. No production construction drawings have been supplied or executed in this session.

To execute real DWG/DXF station acceptance on a machine with drawings, open PowerShell at the repository root:

```powershell
cd CADTBT360_V1.000
$env:CADTBT360_ROAD_DXF_FIXTURE = "D:\\CAD-Test\\Road-Plan.dxf"
$env:CADTBT360_ROAD_DWG_FIXTURE = "D:\\CAD-Test\\Road-Plan.dwg"
$env:CADTBT360_ROAD_EXPECTED_STATIONS = "Km1+000;Km1+020;Km1+040"
cargo test --locked -p cadcraft --bin cadcraft actual_dwg_dxf_station_fixture_acceptance -- --ignored --nocapture
```

The above drawing paths and stations are EXAMPLES, not claims of inspected files.

## 4. Verification and packaging separation
- Feature test workflow: .github/workflows/cadtbt360-road-qto-feature-tests.yml
- It runs road quantity tests, layout tests and cargo check on a Windows runner. It intentionally performs no portable EXE build or Inno Setup packaging.
- The prior Windows release workflow V1.023 run #7 succeeded (commit 878897c6); that success does NOT validate new commits on this feature branch.
- Packaging must be scheduled only after feature CI checks and the relevant real drawing acceptance have been reviewed.
