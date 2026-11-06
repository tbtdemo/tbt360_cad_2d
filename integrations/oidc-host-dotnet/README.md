# CAD 360 — ASP.NET Core JWT/OIDC verified-host reference (Stage249)

This is a **secure authentication boundary prototype**, NOT an operational ChatGPT/Gemini-to-CAD bridge.

## Requirements

- .NET 8 SDK; ASP.NET Core runtime.
- AuthServer (e.g. TBT OpenIddict) with a real HTTPS OIDC issuer and JWKS endpoint.
- A valid OAuth2 access token signed by AuthServer for an exact `aud` accepted by CAD 360.
- JWT with verified `sub`, `tenant_id`, and `role` claims (`viewer`, `editor`, or `approver`). Map your existing AuthServer claim names explicitly; never silently substitute a claim from client-provided JSON.

## Local setup

PowerShell:

```powershell
$env:CAD360_OIDC_AUTHORITY = "https://YOUR-AUTH-SERVER-ISSUER"
$env:CAD360_OIDC_AUDIENCE = "cad360-mcp"
dotnet restore .\integrations\oidc-host-dotnet\Cad360OidcHost.csproj
dotnet run --project .\integrations\oidc-host-dotnet\Cad360OidcHost.csproj
```

Call `GET http://127.0.0.1:5049/api/cad360/identity` with `Authorization: Bearer <access_token>`. Invalid signature, issuer/audience, expiry, role, tenant, or anonymous calls must be rejected. A successful response confirms verified identity **only**; it reports `mcpSessionCreated:false`.

Do not accept a raw JWT payload parsed without signature verification; do not use a chat-app session cookie as a CAD OAuth credential.

## Security architecture

- ASP.NET JwtBearer validates signature via OIDC metadata and JWKS, issuer, audience, lifetime, and signing algorithm policy set by the host.
- The Rust MCP policy `OidcTrustPolicy` receives an already verified identity through a **trusted in-process** `VerifiedTokenProvider`. It independently checks expected issuer, audience, exp/nbf, role, tenant, and per-tool grants. An arbitrary JSON-RPC caller is never a trusted provider.
- **Not implemented**: a secure authenticated host↔MCP channel joining the ASP.NET host sample to Rust `Server::with_verified_oidc`, token revocation/introspection, multi-tenant database Row-Level Security, production key rotation tests, end-to-end host integration, Windows EXE acceptance.
- The host sample intentionally does not expose arbitrary `tools/call`, file paths, or a generic CAD command endpoint.
- Run only on loopback. For any future network deployment require HTTPS, authentication, per-user/Tenant authorization, rate limits, audit, and origin controls.

## Test criteria

1. Missing token -> 401.
2. Bad signature or wrong issuer/audience -> 401.
3. Expired/not-yet-valid token -> 401.
4. Signed token without supported role/tenant/user -> 403.
5. Viewer can read scoped drawing only **after** the future trusted MCP bridge is implemented.
6. Invalid Tenant, UID, or expired scoped session -> denied at MCP Server.
7. No claims, token, or customer DWG content in audit logs.

Status: source/template provided; no live AuthServer tokens, signature-verification integration log, or Windows build PASS has been supplied. Do not treat this sample as proof of end-to-end OIDC.


## Stage250 — Authenticated read-only MCP to running desktop CAD

The host now exposes `GET /api/cad360/mcp/read-drawing` with the same JWT bearer authorization as the identity endpoint. This endpoint performs a **read-only** `inspect_drawing` MCP call against the running CAD desktop's loopback control service using the CAD CLI.

Host setup, PowerShell:

```powershell
$env:CAD360_OIDC_AUTHORITY = "https://YOUR-AUTH-SERVER-ISSUER"
$env:CAD360_OIDC_AUDIENCE = "cad360-mcp"
$env:CAD360_LOCAL_TENANT_ID = "YOUR-TRUSTED-TENANT-ID"
$env:CAD360_MCP_CLI_PATH = "C:\\path\\to\\cadcraft-cli.exe"
$env:CAD360_MCP_CONNECT_ADDRESS = "127.0.0.1:7979"
dotnet run --project .\\integrations\\oidc-host-dotnet\\Cad360OidcHost.csproj
```

In the *same desktop user account* start CAD 360 with its protected local control server available. The CLI must support `cadcraft-cli mcp --connect 127.0.0.1:7979` and use the existing private per-user loopback control-token file.

The endpoint denies access when the JWT has no valid `sub`/`tenant_id`/`role`, the verified JWT tenant does not match `CAD360_LOCAL_TENANT_ID`, the executable path is not absolute and present, or the control target is not IPv4 loopback. The host does not accept a filename, shell command, tool name or TCP address from HTTP request parameters. It launches the CLI with `CAD360_MCP_ACCESS_MODE=readonly`, completes MCP initialize/initialized, invokes only `inspect_drawing`, then terminates the subprocess (10-second maximum).

**Scope limitation:** This enables verified OIDC host -> local read-only MCP bridge **in source**, not a signed JWT-to-Rust credential exchange. The Rust MCP subprocess itself is limited to read-only tools and the local desktop control token; tenant enforcement is at the authenticated .NET endpoint, not in Rust. It is not a cross-tenant shared-host service or a safe generic MCP proxy. Do not expose the sample beyond loopback or use it with concurrent cross-tenant CAD desktop sessions.

To run unit-style policy checks without credentials or a CAD process:

```powershell
dotnet run --project .\\integrations\\oidc-host-dotnet\\Cad360OidcHost.csproj -- --self-test
```

Expected: checks for same-tenant viewer/editor, different tenant denial, missing identity, bad role and non-loopback addresses. This does **not** test JWKS signatures or desktop connectivity. Those require the actual AuthServer, desktop EXE and CI acceptance.

## Stage251 — Authenticated AI drawing PREVIEW (no write)

Host endpoint `POST /api/cad360/mcp/ai-preview` takes only `{"text":"vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 0,0"}` from an authenticated Bearer JWT. The host requires same `tenant_id` as `CAD360_LOCAL_TENANT_ID`, role `editor` or `approver`, and a text length of 1–4000 without control characters.

The endpoint starts `cadcraft-cli mcp --connect 127.0.0.1:7979` using `CAD360_MCP_ACCESS_MODE=reviewed`. It invokes **only** `ai_draw_plan_vietnamese` with `preview:true`, `confirm:false`, `createDoorBlock:false`, `applyLayers:false`. Arbitrary MCP tool names, DWG paths, shell commands and `confirm:true` are **not** accepted in HTTP parameters. Viewer, invalid tenant and oversized prompts are rejected. The process is terminated after the call; timeout is 10 seconds.

Example request (with valid AuthServer access token):

```powershell
$body = @{ text = "vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 0,0" } | ConvertTo-Json
Invoke-RestMethod -Method Post -Uri http://127.0.0.1:5049/api/cad360/mcp/ai-preview -Headers @{ Authorization = "Bearer $token" } -ContentType application/json -Body $body
```

**There is intentionally no AI commit/execute endpoint in this release.** Preview/Confirm/Undo and document UID/epoch/revision protections will need a separate, tested, explicitly user-approved write workflow. Run `dotnet run --project integrations/oidc-host-dotnet/Cad360OidcHost.csproj -- --self-test` for host role and prompt guards. This test does not prove that the live CAD desktop or AuthServer were contacted. Do not claim end-to-end acceptance without actual CI, signed token and EXE test evidence.
