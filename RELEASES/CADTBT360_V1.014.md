# CADTBT360_V1.014 — explicit Km parsing in CAD annotations

Branch: release/CADTBT360_V1.014
Based on: release/CADTBT360_V1.013
State: SOURCE COMMITTED. CI PASS, Windows EXE and Setup are NOT YET VERIFIED.

## Completed code changes
- Extend explicit Km station detection from CAD Text, MText and Block attribute/definition labels to accept trailing punctuation on station tokens, e.g. "Km1+025.50." and "Km1+025.50,".
- Keep strict evidence safety: free-text station numbers without Km and arbitrary word suffixes remain rejected; multiple conflicting Km station values still require manual review.
- Add Rust regression assertions covering both punctuation forms and invalid arbitrary letter suffix.
- Increment desktop release label, QTO screen heading and default Excel workbook filename to CADTBT360_V1.014.
- Create Windows x64 workflow for release V1.014 including a dedicated Km annotation parsing regression gate, existing AEA/marker/area tests, Cargo compile, portable-first artifact stage, Inno Setup and Windows install/uninstall smoke steps.

## Still open
- No successful GitHub Windows build/test status verified for this release. Merely committing a workflow does not mean any EXE exists.
- Multi-layout station detection and geometry-based cut/fill transition interpolation remain pending.
- Source Cargo workspace remains in the historical CADTBT360_V1.000 directory.
- Native DXF/DWG QTO comparison, Excel printed formatting and actual GUI startup were not tested.

## Intended build outputs (only when workflow PASS)
CADTBT360_V1.014.exe
CADTBT360_V1.014_Portable.zip
CADTBT360_V1.014_Portable_SHA256.txt
CADTBT360_V1.014_Setup.exe
CADTBT360_V1.014_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1014-windows.yml
