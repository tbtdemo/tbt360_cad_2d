# CADTBT360_V1.007 — Windows packaging hardening and station handle audit

Branch: release/CADTBT360_V1.007
Parent release: release/CADTBT360_V1.006
Status: SOURCE COMMITTED. Windows build, EXE and setup NOT VERIFIED.

## Actual code changes
- Reject duplicate CAD plan-marker handles before generating road Excel, including case and leading zero differences.
- Normalize marker handles for linking, scan UI and audit; add regression tests for reused and zero-padded references.
- Preserve V1.006 live station label and geometry validation.
- Update CAD window version and default workbook filename to V1.007.
- Windows workflow validates locked Cargo workspace metadata, runs road and marker tests, builds release EXE, packages portable ZIP, Inno Setup EXE and SHA256 manifest.
- Use stable Inno Setup AppId {6BCA4011-C394-4C3D-AB6D-3D8D5A0B1000}; older per-version installed copies may require manual uninstall.

## Acceptance criteria NOT yet achieved
- No Windows-native cargo test/build PASS verified.
- No setup tested with actual CAD model or Excel Print Preview.
- No confirmed downloadable EXE, ZIP or installer artifacts.
- Cargo source workspace retains historical CADTBT360_V1.000 folder name.
- Road cut-fill zero-crossing interpolation and multi-layout scan remain unfinished.

## Expected outputs after successful CI
CADTBT360_V1.007.exe
CADTBT360_V1.007_Portable.zip
CADTBT360_V1.007_Setup.exe
CADTBT360_V1.007_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1007-windows.yml
