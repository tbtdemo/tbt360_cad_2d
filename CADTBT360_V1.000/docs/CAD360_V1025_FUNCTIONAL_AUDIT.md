# CAD 360 V1.025 — Functional completion and release gates

Updated: 2026-11-04. This is a source audit; NOT a certification of features in a shipped executable.

## Evidence rules

- **Implemented** = source implementation found; not necessarily tested in a packaged Windows EXE.
- **CI PASS** = an identified GitHub Actions run executed the specified tests and succeeded.
- **Release PASS** requires a new Windows GUI/CLI compiled from the same exact commit, an installed Setup and a Portable bundle, checksums, and a GUI smoke test.
- Never report a percentage without a counted acceptance matrix. Never treat a test-only commit as a shipped installer.

## High-priority gaps and acceptance criteria

| Priority | Area | Current evidence | Remaining acceptance gate |
| --- | --- | --- | --- |
| P0 | Native DWG preservation | `docs/gaps.md` FF1 records loss of unsupported custom classes, objects, xdata and reactors through the DWG bridge | Use an independently sourced corpus of genuine DWGs; compare supported and unsupported records before/after two save-reopen cycles; stop unsafe silent overwrites when unsupported payloads are present |
| P0 | AutoCAD interoperability | Synthetic DWG R2018 Xref roundtrips and 10k/50k/100k geometry tests passed in run 41813296690 | Open, edit, save and reopen real drawings in AutoCAD and CAD 360; check actual dimensions, layouts, block attributes and xrefs |
| P0 | Drawing Recovery | Desktop autosave/recovery exists; one-click irreversible Discard was protected in commit 1d79de69 | Desktop Reliability Windows build and GUI test; ensure opening a recovered drawing does not delete its last recovery copy prematurely |
| P1 | XREF/XCLIP/REFEDIT | Xref attach/reload/unload/bind; rectangular XCLIP; in-place editing of ordinary block references | Polygonal XCLIP, nested picking, external-source writeback and attribute synchronization |
| P1 | Dynamic Blocks | Read/edit supported parameters; validation | Authoring tools for parameters/actions and roundtrip of imported dynamic blocks |
| P1 | MTEXT | Existing on-canvas editing and formatting tools | Edit preexisting MTEXT, preserve formatting, fields and stacked fractions |
| P1 | Dimension | Core dimension types with DimStyle variables | Complete DimStyle manager and associative updates for advanced edits |
| P1 | Hatch | Patterns and property editing | Preview, curved boundaries, more patterns and robust island behavior |
| P1 | Plot/PDF | Plot dialog, PDF output, style table support | Binary CTB/STB/PC3 interoperability and a physical printer path |
| P2 | AutoLISP | Core interpreter, commands and script support | DCL/Visual LISP/reactors and compatibility suite |
| P2 | AI/MCP | Command execution, CLI, remote/headless and PNG rendering | Multi-step agent acceptance on the exact installed Windows EXE; report tool capabilities honestly |
| P2 | 3D | Limited geometry and visual style metadata | 3D renderer, solids, surfaces and inspection/UI workflows |

## Release V1.025 quality gates

1. `cad360-v1025-xref.yml`: Xref DXF and DWG plus native large-DWG integrity.
2. `cad360-v1025-desktop-reliability.yml`: compile/test desktop recovery UI.
3. `cad360-v1025-release.yml`: build from release SHA, install/uninstall on Windows, validate Setup/Portable SHA256.
4. Manual or separately automated GUI acceptance (open files, save, Undo/Redo, panels, text and sample Xrefs) on the **packaged V1.025** executable, not on source-only CLI tests.
5. Verify native DWG compatibility with real drawings before claiming general AutoCAD compatibility.

## Current deployment note

Do not label V1.025 stable until the packaging and UI acceptance jobs succeed. Do not increment V1.026 until a distinct newer installer build is actually made.
