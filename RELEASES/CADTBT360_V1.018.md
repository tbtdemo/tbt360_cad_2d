# CADTBT360_V1.018 — Source marker type validation and Windows installer exit status

Branch: release/CADTBT360_V1.018
Parent: release/CADTBT360_V1.017
Status: Source committed; Windows build, EXE and Setup artifacts not independently verified.

## Changes in V1.018
- Reject road plan markers that are still present under the same CAD handle but no longer represent TEXT, MTEXT or INSERT. Previously the export path skipped validating the station text when an entity had changed kind.
- The on-screen "KIỂM TRA NGUỒN CỌC TRƯỚC KHI XUẤT" audit now counts markers whose entity type changed as altered sources.
- The road Excel export still verifies exact station chainage for supported TEXT/MTEXT/INSERT markers and confirms geometric area sources are current.
- Increment desktop version, quantity report title and default Excel filename to CADTBT360_V1.018.
- Create dedicated Windows workflow for V1.018 with existing Cargo tests, release build, early portable artifact upload, PE and ZIP checks and Inno Setup.
- Windows installer smoke check now uses Start-Process -Wait -PassThru for Setup and Uninstaller to avoid checking output before those processes finish and to capture their exit codes.

## Pending verification
- The connected GitHub tool set does not include a workflow-dispatch action; writing workflow source alone is not proof of CI execution.
- No successful Windows build, startup smoke test, downloaded EXE, portable ZIP or installer verified in this turn.
- DXF/DWG benchmark, Excel print acceptance, multi-layout station scanning and cut/fill transition calculation remain open.

Expected artifacts after successful workflow:
CADTBT360_V1.018.exe
CADTBT360_V1.018_Portable.zip
CADTBT360_V1.018_Portable_SHA256.txt
CADTBT360_V1.018_Setup.exe
CADTBT360_V1.018_SHA256.txt

Workflow: .github/workflows/cadtbt360-v1018-windows.yml
