# CADTBT360_V1.011 — Correct station conflict regression and Windows build gates

Source branch: release/CADTBT360_V1.011
Parent: release/CADTBT360_V1.010
State: CODE COMMITTED. Windows CI and downloadable EXE not yet verified.

## Corrections confirmed in GitHub source
- Fixed an incorrect test expectation in `zero_padded_hex_marker_is_considered_the_same_entity`: when a scan finds a label at the correct chainage but the object handle is already used by another section, result is (linked=0, missing=0, conflicts=1); previous assertion expected (0,1,1) and would fail if the code behaved correctly.
- Added separate test for truly absent station: expected (0,1,0), not a conflict.
- Desktop title, runtime version and road Excel default file name increased to CADTBT360_V1.011.
- Dedicated Windows workflow runs marker regression tests, existing AEA and station validation tests, checks Cargo and compiles release.
- Existing portable-first EXE and ZIP upload, SHA-256, PE validation, Inno Setup smoke install/uninstall retained.

## Verification limitations
- No GitHub Actions run PASS, actual Windows installer, GUI screenshot or direct-download artifact verified by these changes.
- Release cannot be accepted merely because the workflow file exists.
- Multi-layout station detection and surveyed cut/fill transition interpolation remain pending.
- Cargo workspace historical folder CADTBT360_V1.000 remains unchanged; output release naming is V1.011.

## Expected outputs after Windows workflow success
CADTBT360_V1.011.exe
CADTBT360_V1.011_Portable.zip
CADTBT360_V1.011_Portable_SHA256.txt
CADTBT360_V1.011_Setup.exe
CADTBT360_V1.011_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1011-windows.yml
