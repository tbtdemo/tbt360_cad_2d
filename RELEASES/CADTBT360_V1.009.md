# CADTBT360_V1.009 — Windows installer smoke test and packaging verification

Branch: release/CADTBT360_V1.009
Parent: release/CADTBT360_V1.008
Status: SOURCE COMMITTED; workflow success and actual download not yet verified.

## Implemented in source/workflow
- Desktop title, road takeoff UI and default Excel workbook filename increment from CADTBT360_V1.008 to CADTBT360_V1.009.
- Windows Rust build script now rebuilds Windows version resources when release identifier or embedded SHA changes; resource FileDescription includes the requested TBT release.
- Dedicated Windows workflow still runs locked Cargo metadata, AEA and marker regression tests before compiling a release Windows executable.
- Portable stage checks Windows MZ and PE signatures, ZIP contents, sizes, and compares extracted ZIP EXE hash with the original compiled release EXE.
- Portable EXE and ZIP are uploaded immediately following successful compilation and verification, even if subsequent Setup creation fails.
- Inno Setup stage creates versioned installer, then silently installs on the Windows runner, verifies installed EXE hash matches the portable EXE, checks presence of uninstall program and silently uninstalls.
- SHA256 manifest is generated for verified EXE, portable ZIP and installer.
- Release documentation change also triggers the workflow on the release branch.

## Outstanding gates
- The existence of this workflow is NOT evidence of a successful Windows build or a downloadable EXE.
- Install/uninstall smoke steps are configured but not confirmed passed.
- GUI open/close, DWG/DXF road project validation, installed app UI screenshots, Excel print checks remain pending.
- Multi-layout station scan and cut/fill transition interpolation remain unfinished.
- Historical Cargo workspace directory CADTBT360_V1.000 is retained to preserve paths; binary version is V1.009.

Expected outputs after successful workflow:
- CADTBT360_V1.009.exe
- CADTBT360_V1.009_Portable.zip
- CADTBT360_V1.009_Setup.exe
- CADTBT360_V1.009_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1009-windows.yml
