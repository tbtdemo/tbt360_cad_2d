# CADTBT360_V1.006 — Verify station source integrity before QTO export

**Status:** CODE COMMITTED; Windows CI and installer unverified.
**Base:** release/CADTBT360_V1.005
**Branch:** release/CADTBT360_V1.006

## Implemented
- Validate the current station label in CAD TEXT/MTEXT/INSERT against the originally linked chainage before exporting Excel. Reject deleted/invalidated or edited station labels rather than producing volumes with stale evidence.
- Keep manually attached non-label CAD marker references compatible with legacy data, but still require these objects to exist.
- Replace panic-prone road interval endpoint unwrap calls with explicit errors.
- Add **KIỂM TRA NGUỒN CỌC TRƯỚC KHI XUẤT** audit button for missing marker bindings, changed/deleted source labels, and duplicate handles.
- Update displayed desktop version, takeoff report heading, XLSX filename to V1.006.
- Add Windows x64 workflow with road takeoff and station matching regression test filters plus release build and Inno Setup portable/installer artifact upload.

## Outstanding acceptance gates
1. GitHub Actions Windows job must execute and pass; a workflow file is not a successful build.
2. Verify versioned portable executable starts and can open actual DWG/DXF drawings.
3. Verify release setup EXE installs and uninstalls on Windows x64 and its icon launches V1.006.
4. Compare AEA Excel reports against a surveyed sample project; verify changed station marker blocks publishing.
5. No algorithm for inferring zero-crossing position between two survey stations is implemented yet.
6. Multi-layout and nested XREF station scanning remain pending.
7. The Cargo workspace source directory still carries its historical CADTBT360_V1.000 name; output names are versioned.

Expected artifact names only on workflow success:
- CADTBT360_V1.006.exe
- CADTBT360_V1.006_Portable.zip
- CADTBT360_V1.006_Setup.exe

Workflow: .github/workflows/cadtbt360-v1006-windows.yml
