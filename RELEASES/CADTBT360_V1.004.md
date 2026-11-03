# CADTBT360_V1.004 — Auto scan road station labels in Model Space

**Build status:** CODE UPDATED, Windows native tests/installer not yet verified.
**Base:** release/CADTBT360_V1.003
**Current branch:** release/CADTBT360_V1.004

## Shipped in source
- QTO button **QUÉT CỌC TOÀN BỘ MODEL** scans existing model-space annotations without requiring preselection.
- Recognition supports TEXT, MTEXT, INSERT block attributes and direct TEXT/MTEXT members of a block definition.
- Parsing requires explicit Km station notation and does not infer chainage from XY drawing coordinates.
- A selected-region scan is still available. Both paths use one station matching routine.
- Markers with missing, duplicate or conflicting station assignments are flagged; manually linked station markers are preserved.
- Full-model scan is deliberately bounded at 100,000 CAD entities and visibly warns if only a partial document was scanned.
- Scanning does not approve cross-sections, invent missing cut/fill areas, or create positive earthwork volumes.
- Desktop title, QTO heading and default XLSX report name are CADTBT360_V1.004.
- Dedicated GitHub Actions Windows x64 workflow for cargo road tests, release EXE, portable ZIP and Inno Setup EXE.

## Pending before acceptance
- Real Windows cargo test PASS and Windows native installer build.
- Verify scanner on authentic road DWG/DXF with mixed annotation styles and multiple layouts.
- The new full-model scan is triggered by a UI button, not automatically during pan/zoom.
- Text in nested block references, attribute formatting variants and non-model-space layouts remain unsupported.
- True zero-crossing interpolation of excavation/fill inside a segment, if required by survey specification, remains pending.
- Source workspace directory retains CADTBT360_V1.000 for Cargo path compatibility; binary/installer output is versioned V1.004.
- Do not call the release stable or distribute as tested without installer and Excel Print Preview verification.

## Expected workflow outputs (only if CI succeeds)
CADTBT360_V1.004.exe
CADTBT360_V1.004_Portable.zip
CADTBT360_V1.004_Setup.exe

Workflow: .github/workflows/cadtbt360-v1004-windows.yml
