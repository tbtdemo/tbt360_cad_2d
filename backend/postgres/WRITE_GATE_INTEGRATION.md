# CAD360 server write-gate integration contract

Migration: `backend/postgres/migrations/007_bim_server_write_gate.sql`

## Required API operation

1. Validate the TBT.Auth OpenIddict access token (issuer, audience, expiration, signature) and map the *validated subject* to an account UUID.
2. Resolve the project from the authorized route, not from an untrusted request body.
3. Open a DB transaction using a **server-only** connection; desktop/browser clients must not have DB credentials.
4. Call `SELECT bim_update_design_object(@project_id,@object_id,@authenticated_account_id,@expected_revision,@geometry_json::jsonb)`.
5. Map SQLSTATE `42501` to HTTP 403; `P0002` to 404; `40001` to 409; `55P03` to 423; `22023` to 400. Roll back transaction for any failure.
6. Only return the new revision after transaction COMMIT. Broadcast updates after successful commit.
7. Do not grant direct INSERT/UPDATE/DELETE on BIM business tables to desktop users or the public network. Restrict EXECUTE on the function to the trusted API role; otherwise an untrusted caller could forge the account argument.
8. Validate permissions again for every CREATE, DELETE, MOVE, IMPORT, XREF, bulk edit and upload path. The new function currently protects only geometry UPDATE.

## Transaction/race acceptance cases

- Authenticated viewer UPDATE -> 403; revision and audit unchanged.
- Different floor/discipline scope without an applicable grant -> 403.
- Explicit deny matching the edited scope -> 403 even if a broad allow exists.
- Wrong project/object pair -> 404; never leak another project's geometry.
- Two editors submit base revision 7 concurrently -> exactly one commits revision 8; other gets 409.
- Other editor holds unexpired lease -> 423; geometry unchanged.
- Failed JSON validation -> 400; geometry unchanged.
- Success increments project revision and object revision and appends an audit entry atomically.
- 5 real clients over HTTP, a reconnect and an interrupted transaction are **not yet verified**.

This is integration code, NOT a claim of production-ready identity enforcement or crash recovery.
