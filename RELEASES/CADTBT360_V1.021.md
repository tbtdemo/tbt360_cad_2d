# CADTBT360_V1.021 — Windows packaging diagnostics

Parent: release/CADTBT360_V1.020
Branch: release/CADTBT360_V1.021
State: SOURCE COMMITTED. Windows CI PASS and real EXE/Setup artifacts NOT YET VERIFIED.

## Changes
- Added strict preflight checks that the Rust workspace Cargo.toml/Cargo.lock and desktop manifest/main source exist on the runner.
- Printed cargo/rustc versions, runner target and commit SHA so failures can be diagnosed from Actions logs.
- Fixed cargo metadata PowerShell pipeline masking a nonzero Cargo exit code.
- Made Windows PE header inspection use the resolved absolute path explicitly.
- Added a failure diagnostics step summarizing Cargo/Rust versions and which portable/installer artifacts exist after failure.
- Kept existing Rust test groups, Windows release build, portable-first artifact upload, Inno Setup installer creation, install/uninstall checks and SHA256 manifest.
- Incremented desktop title, QTO heading and default Excel export name to CADTBT360_V1.021.

## Important status
- GitHub workflow source was committed, but a workflow run has not been independently verified PASS.
- Do not report installer as usable without the run ID, passing jobs and artifact list.
- GitHub Actions link: https://github.com/tbtdemo/tbt360_cad_2d/actions/workflows/cadtbt360-v1021-windows.yml
- The connected GitHub interface currently cannot dispatch a workflow or enumerate push-triggered runs by commit. Legacy combined commit statuses alone are not Windows Actions success evidence.

## Pending product and acceptance work
- DWG/DXF reference drawing acceptance, actual desktop GUI launch and Excel page layout checks.
- Scan station annotations across multiple Layouts and correctly handle cut-to-fill transition points.
- Downloadable Windows x64 executable and Inno Setup acceptance.

Expected outputs only if workflow succeeds:
CADTBT360_V1.021.exe
CADTBT360_V1.021_Portable.zip
CADTBT360_V1.021_Portable_SHA256.txt
CADTBT360_V1.021_Setup.exe
CADTBT360_V1.021_SHA256.txt
