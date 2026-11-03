# CADTBT360_V1.008 — Windows portable-first packaging and build identity

Status: CODE COMMITTED; Windows compilation, tests, portable EXE, and installer not verified.
Parent: release/CADTBT360_V1.007
Branch: release/CADTBT360_V1.008

## Changes in source
- Desktop title, status label, QTO screen and default Excel workbook name incremented to CADTBT360_V1.008.
- Version diagnostics display separate TBT release label and underlying Cargo engine version. CI embeds both the release name and commit SHA.
- Dedicated Windows workflow validates Cargo lock, runs road QTO and station matching tests, and compiles a release build.
- Before publishing portable artifacts, the workflow verifies the expected EXE and ZIP exist, checks plausible file sizes and validates the EXE MZ signature.
- Immediately uploads CADTBT360_V1.008.exe and CADTBT360_V1.008_Portable.zip after successful compilation, *before* Inno Setup. Thus installer failures do not discard an already compiled portable EXE.
- After setup succeeds, uploads setup EXE and SHA256 checksums. A stable installer AppId is retained from V1.007.
- Prior road marker deduplication and live source chainage verification are retained.

## Remaining failures / acceptance criteria
- No Windows cargo test/build PASS was confirmed through GitHub checks. This workflow needs an actual Windows run to prove it works.
- No generated setup file is available for direct download in this interaction; links to workflow are not artifact downloads.
- No native GUI smoke test or DWG/DXF takeoff validation is claimed.
- Cargo workspace directory still uses legacy CADTBT360_V1.000 name.
- Road transition interpolation between cọc, automatic cross-section classification and cross-layout scanning remain unimplemented.

## Expected files once build succeeds
- CADTBT360_V1.008.exe
- CADTBT360_V1.008_Portable.zip
- CADTBT360_V1.008_Setup.exe
- CADTBT360_V1.008_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1008-windows.yml
