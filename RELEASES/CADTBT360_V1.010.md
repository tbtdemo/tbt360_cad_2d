# CADTBT360_V1.010 — 10-version checkpoint and Windows packaging gate

Branch: release/CADTBT360_V1.010
Parent: release/CADTBT360_V1.009
Status: SOURCE COMMITTED. Windows CI PASS and installable EXE NOT YET VERIFIED.

## Work included in V1.010
- Correct nested block station label traversal: the CAD doc block's EntityStore has an iter() method but no reference-based IntoIterator; code now uses def.entities.iter() to avoid a Rust compile-time iterator error.
- Update user-visible title, release diagnostics, QTO report heading, and XLSX default filename to CADTBT360_V1.010.
- Dedicated Windows workflow first checks locked Cargo metadata, runs AEA and station matching tests, then performs cargo check and release cargo build.
- Verify compiled native executable exists; validate Windows PE and portable ZIP contents, and upload versioned portable EXE/ZIP together with portable SHA256 before Inno Setup.
- Build installer with Inno Setup and smoke test its silent install/uninstall and installed file SHA256 on windows-latest, then upload full release package checksums.

## Checkpoint scope inherited from V1.001–V1.009
- Station identity links from plan markers to cross sections; explicit Km parsing in TEXT/MTEXT/Block including nested definitions, missing and duplicate checks.
- Average End Area calculation for cut/fill and explicitly surveyed zero-area station endpoints; road interval filtering and source-geometry evidence.
- Configurable takeoff workbook selection, default Excel naming, CAD source evidence checking and protected linked marker handling.
- All inherited functionality requires actual native builds and reference drawing acceptance testing; no release PASS inferred.

## Unfinished / blockers
- GitHub Actions must actually run and report success; workflow configuration alone is not build evidence.
- Native GUI launch, road DWG/DXF validation, Excel print and installed application screenshots not yet confirmed.
- Multi-layout automatic station scanning and cut/fill transition point interpolation are not implemented.
- Source workspace keeps historical CADTBT360_V1.000 folder path for Cargo compatibility; the published V1.010 version is separate.

## Expected GitHub Actions output, only after CI success
- CADTBT360_V1.010.exe
- CADTBT360_V1.010_Portable.zip
- CADTBT360_V1.010_Portable_SHA256.txt
- CADTBT360_V1.010_Setup.exe
- CADTBT360_V1.010_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1010-windows.yml
