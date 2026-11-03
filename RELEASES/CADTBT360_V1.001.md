# CADTBT360_V1.001 — Windows x64 release candidate

Version: CADTBT360_V1.001
Previous: CADTBT360_V1.000
Status: SOURCE UPDATED; WINDOWS BUILD / INSTALLER NOT YET VERIFIED

## Scope
- Attach an explicitly selected plan station marker (CAD entity handle) to each road cross-section.
- Persist marker handles in the road_takeoff JSON, retaining compatibility with V1.000 sections that lack this optional field.
- Reject reused marker handles when storing another station.
- Show marker handle in the cross-section table and perform one-to-one validation of plan markers and stationing.
- Preserve current AEA cut/fill volumes, chainage-range filtering and explicit source validation.
- Display CADTBT360_V1.001 in the desktop title and version label.

## Known gaps / next version
- Automatic station-marker text/attribute recognition and drawing geometry association are not implemented.
- Zero cut/fill transition points need their own explicit surveyed zero-area state.
- Windows-native CI pass, performance and Excel Print Preview are required for acceptance.
- Existing Rust workspace folder CADTBT360_V1.000 stays in place to avoid breaking Cargo relative paths; it is not the V1.001 executable or setup name.

## Windows build artifacts when GitHub Actions succeeds
- CADTBT360_V1.001.exe (portable)
- CADTBT360_V1.001_Portable.zip
- CADTBT360_V1.001_Setup.exe (Inno Setup installer)

Workflow: .github/workflows/cadtbt360-v1001-windows.yml
Release branch: release/CADTBT360_V1.001

## Versioning policy
Increment released candidate version: V1.001 → V1.002 → V1.003. Keep original source history and mandatory third-party licenses. Do not mark release delivered until the exact-version Windows EXE and setup are built and checked.
