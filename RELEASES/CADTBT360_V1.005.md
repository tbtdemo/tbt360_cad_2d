# CADTBT360_V1.005 — nested road station blocks and safer model scanning

Status: SOURCE COMMITTED — Windows CI, EXE and installer NOT YET VERIFIED.
Base branch: release/CADTBT360_V1.004
Current branch: release/CADTBT360_V1.005

## Code changes
- Recursively read TEXT/MTEXT from nested INSERT block definitions and visible INSERT attributes.
- Preserve the outer selected INSERT handle as station evidence.
- Stop recursion after 12 levels and reject cyclic block definitions.
- If the same block holds multiple distinct explicit Km chainages, reject instead of selecting the first.
- Reject automatic full-model linking beyond 100,000 CAD entities, rather than linking incomplete results.
- Add station matching tests for exact matching, reused handles, and duplicate chainage conflicts.
- Change desktop title, QTO report heading, and Excel default filename to CADTBT360_V1.005.
- Add Windows workflow for road module tests and Windows EXE + Inno Setup outputs.

## Remaining work / acceptance
- Complex dynamic blocks, nested xrefs, non-model layouts, virtual annotation transforms and geometry-based cross-section discovery remain unfinished.
- Accurate excavation/fill transition interpolation between surveyed stations is not implemented.
- Real Windows build/tests have not been confirmed. XLSX print preview and DWG/DXF measurements require checking with representative user drawings.
- Cargo source workspace path still contains CADTBT360_V1.000; this is a source workspace name, not the published version.
- The workflow is a packaging definition, NOT evidence of completed EXE or Setup.

## Intended packaging output, only after successful GitHub Actions execution
CADTBT360_V1.005.exe
CADTBT360_V1.005_Portable.zip
CADTBT360_V1.005_Setup.exe

Workflow: .github/workflows/cadtbt360-v1005-windows.yml
