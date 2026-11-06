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
