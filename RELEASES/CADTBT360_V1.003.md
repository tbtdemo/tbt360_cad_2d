# CADTBT360_V1.003 — Scan selected station labels

**Status:** CODE COMMITTED; WINDOWS TEST/EXE NOT YET VERIFIED

## Implemented
- Dedicated release branch based on CADTBT360_V1.002.
- CAD QTO button **QUÉT NHÃN CỌC TỪ VÙNG CHỌN** reads only actively selected CAD objects, not the entire project.
- Supports TEXT, MTEXT and INSERT block attributes with explicit Km stationing.
- Compares marker stationing to recorded road cross-sections within 0.001 m; reports unmatched or ambiguous stations separately.
- Does not overwrite a conflicting existing station marker, automatically confirm survey records, or infer labels from drawing coordinates.
- Rejects a block with multiple conflicting Km station labels.
- Normalizes MTEXT paragraph separators for explicit Km labels; fixes a malformed Rust character literal from V1.002.
- Version displays and default Excel filename use CADTBT360_V1.003.
- Windows x64 build/test, portable ZIP and setup workflow configured.

## Still unfinished
- Automatically start a canvas selection rectangle and read station labels without a previously selected set.
- Read TEXT/MTEXT inside block definitions without attributes; currently only INSERT attributes are parsed.
- Scan multiple layouts automatically, annotate or highlight recognized stations on the CAD viewport, and propose unmatched pairings.
- Infer a cut/fill zero-crossing location between two surveyed cross-sections from supported geometry. Existing zero-area support still requires verified measurements.
- Confirm Windows compile/test pass, installer execution, real drawing performance, and Excel Print Preview.

## Files and release gates
- Source workspace is retained in CADTBT360_V1.000/ for Cargo path compatibility. This path is not the new executable version.
- Workflow: .github/workflows/cadtbt360-v1003-windows.yml
- Expected only after workflow success: CADTBT360_V1.003.exe, CADTBT360_V1.003_Portable.zip, CADTBT360_V1.003_Setup.exe.
- Release is not accepted until tests pass, installer starts on Windows, and takeoff results match an independently checked drawing.
