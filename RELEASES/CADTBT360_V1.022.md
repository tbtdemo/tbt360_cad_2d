# CADTBT360_V1.022 — Windows EXE identity and installer artifact checkpoint

Branch: release/CADTBT360_V1.022
Parent: release/CADTBT360_V1.021
Status: SOURCE COMMITTED; Windows CI PASS and actual downloadable EXE/Setup not independently verified.

## Actual code/workflow changes
- Increment runtime UI and road takeoff Excel/report identity to CADTBT360_V1.022.
- Require Windows resource embedding during cargo build (CADCRAFT_REQUIRE_WINRES=1), instead of tolerating failures to stamp the EXE with version/icon information.
- Validate compiled cadcraft.exe Windows VersionInfo.FileDescription contains CADTBT360_V1.022 and CompanyName identifies TBT. A stale/unbranded executable fails the release gate.
- Upload an expressly unverified Inno Setup EXE artifact immediately after installer compilation and before installation smoke testing. The user retains an inspectable installer even if the later smoke test fails.
- Keep separately named verified Setup artifact at the end of a successful build/test/install/uninstall pipeline, along with portable EXE/ZIP SHA256 files.
- Retain Cargo metadata checking, road AEA and station linking tests, PE/ZIP inspection, portable-first upload, and detailed GitHub Actions failure diagnostics.

## Acceptance requirements and naming
- CADTBT360_V1.022_Portable_Windows_x64: only appears after successful build and PE/ZIP verification.
- CADTBT360_V1.022_Setup_Unverified_Windows_x64: exists if installer compiled but install smoke may still fail. Do not claim usable.
- CADTBT360_V1.022_Setup_Windows_x64: full package after successful silent setup and uninstall checks, subject to overall workflow success.
- No successful job or artifact has been independently observed for this branch in this session.
- No actual Windows GUI launch, real drawing QTO comparisons or Excel print acceptance tests performed.

## Open features
- Multi-layout automatic station scanning, geometry-grounded cut/fill transition computation, and native CAD/Excel acceptance.
- Cargo source workspace remains CADTBT360_V1.000 by historical design; app version is V1.022.

Workflow: .github/workflows/cadtbt360-v1022-windows.yml
