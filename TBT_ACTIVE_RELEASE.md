# CAD360 TBT — Active release branch

Branch: `tbt/cad360-active-release`

This branch is isolated from `feature/bim-issue-audit` and must produce a single TBT Windows setup. Do not merge historical CAD or QTO reference workspaces into the handover archive.

## Build and delivery

- Direct Windows installer workflow: `.github/workflows/tbt-exe-direct.yml`.
- Portable build and active-source archive: `.github/workflows/cad360-ifc-portable.yml`.
- Target installer: `CAD360_VietNam_TBT_Setup.exe`.
- Active-source ZIP: `CAD360-TBT-Active-Source.zip`.
- Historical directories `CAD_ORIGINAL_014_REFERENCE` and `QTO_R134_WEB_WORKSPACE` are excluded from the handover archive but still exist in the Git tree pending dependency and legal review.
- Core Rust workspace still lives at `SOURCE_CAD_R65` in Git and contains required `cadcraft-*` crates. Never delete them before a passing replacement build.

## Isolation and evidence

1. Use separate build artifacts and SHA-256 manifests per commit.
2. Never label a workflow `queued` as a successful build.
3. A Git branch only isolates source history. Multi-user model edits require server-side authorization, revision compare-and-swap, and project/tenant isolation.
4. Existing deployed user data must not be renamed, removed, or migrated as part of repository cleanup without a verified migration plan.
5. Preserve upstream copyright notices, required license files, and attributions.

## Release gates

- `cargo check --workspace --locked` PASS.
- `cargo test --workspace --locked --no-fail-fast` PASS.
- Windows release compile PASS.
- Inno Setup produces a valid Setup EXE.
- Install, open, save/reopen, uninstall smoke tests PASS on a clean Windows environment.
- Publish checksum and exact commit SHA.
