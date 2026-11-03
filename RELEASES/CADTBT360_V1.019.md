# CADTBT360_V1.019 — KM case-insensitive chainage parsing and Windows package gate

Branch: release/CADTBT360_V1.019
Parent: release/CADTBT360_V1.018
State: CODE COMMITTED; Windows CI PASS and EXE/Setup artifacts NOT VERIFIED.

## Implemented
- Fix station_to_metres parsing to support Km, KM, km and mixed-case kM prefixes consistently for chainage inputs and other station-related workflows.
- Preserve strict handling: reject unrelated prefixes such as XKM12+345 and preserve existing checks for negative stationing, multi-plus, invalid metre ranges and non-finite input.
- Add Rust tests for uppercase and mixed-case Km station inputs, with decimal comma and no plus.
- Increment desktop visible version, road takeoff report heading and Excel default filename to CADTBT360_V1.019.
- New Windows workflow tests this parsing regression before cargo release compilation, portable-first EXE/ZIP upload, PE checks, Inno Setup build and silent install/uninstall verification.

## Verification gates not yet passed
- No successful GitHub Actions Windows run was confirmed in this session; no EXE, portable zip or setup download has been verified.
- An Actions workflow file in a branch does not itself prove successful build or artifact upload.
- Real DWG/DXF quantity acceptance, Excel formatting validation and installed GUI smoke testing remain open.
- Automatic multi-layout plan marker scan and cut/fill transition-point interpolation remain unfinished.
- Source workspace path remains CADTBT360_V1.000 for Cargo compatibility.

## Expected files if Windows CI succeeds
CADTBT360_V1.019.exe
CADTBT360_V1.019_Portable.zip
CADTBT360_V1.019_Portable_SHA256.txt
CADTBT360_V1.019_Setup.exe
CADTBT360_V1.019_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1019-windows.yml
