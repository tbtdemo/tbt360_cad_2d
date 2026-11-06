// CAD 360 verified OIDC host sample. This endpoint validates signatures using
// ASP.NET Core JwtBearer + authority JWKS. It does NOT proxy MCP commands yet:
// a trusted, authenticated in-process bridge is required before granting CAD access.
using System.Security.Claims;
using System.Diagnostics;
using System.Text.Json;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.IdentityModel.Tokens;

var builder = WebApplication.CreateBuilder(args);
string authority = Environment.GetEnvironmentVariable("CAD360_OIDC_AUTHORITY") ?? "";
string audience = Environment.GetEnvironmentVariable("CAD360_OIDC_AUDIENCE") ?? "";
if (!Uri.TryCreate(authority, UriKind.Absolute, out var issuerUri) || issuerUri.Scheme != Uri.UriSchemeHttps) throw new InvalidOperationException("Configure CAD360_OIDC_AUTHORITY as the exact HTTPS issuer.");
if (string.IsNullOrWhiteSpace(audience) || audience.Length > 200) throw new InvalidOperationException("Configure CAD360_OIDC_AUDIENCE.");
builder.WebHost.UseUrls("http://127.0.0.1:5049");
builder.Services.AddAuthentication(JwtBearerDefaults.AuthenticationScheme).AddJwtBearer(opt =>
{
    opt.Authority = authority.TrimEnd('/');
    opt.Audience = audience;
    opt.RequireHttpsMetadata = true;
    opt.MapInboundClaims = false;
    opt.TokenValidationParameters = new TokenValidationParameters
    {
        ValidateIssuer = true, ValidIssuer = authority.TrimEnd('/'),
        ValidateAudience = true, ValidAudience = audience,
        ValidateLifetime = true, RequireExpirationTime = true,
        ValidateIssuerSigningKey = true, RequireSignedTokens = true,
        ClockSkew = TimeSpan.FromSeconds(15),
        NameClaimType = "sub", RoleClaimType = "role"
    };
});
builder.Services.AddAuthorization(options =>
{
    options.AddPolicy("CAD360Scoped", policy => policy.RequireAuthenticatedUser()
        .RequireClaim("sub").RequireClaim("tenant_id").RequireClaim("role"));
});
var app = builder.Build();
app.UseAuthentication();
app.UseAuthorization();
app.MapGet("/health", () => Results.Ok(new { status = "online", mcpProxyEnabled = false }));
app.MapGet("/api/cad360/identity", (ClaimsPrincipal user) =>
{
    var uid = user.FindFirstValue("sub");
    var tenant = user.FindFirstValue("tenant_id");
    var role = user.FindFirstValue("role")?.ToLowerInvariant();
    if (string.IsNullOrWhiteSpace(uid) || string.IsNullOrWhiteSpace(tenant)
        || role is not ("viewer" or "editor" or "approver")) return Results.Forbid();
    return Results.Ok(new { userId = uid, tenantId = tenant, role, verified = true,
        mcpSessionCreated = false, note = "JWT validated by ASP.NET JWT bearer middleware; no CAD session opened." });
}).RequireAuthorization("CAD360Scoped");

/*
 * Stage250: actual authenticated READ-ONLY MCP bridge.
 * The authenticated user must belong to the local tenant configured on this host.
 * CLI is launched with readonly policy and no tool name or shell input from HTTP.
 * No write endpoint is exposed. The CAD desktop loopback has its own per-user
 * control-token protection; the host process must run as the same OS user.
 */
app.MapGet("/api/cad360/mcp/read-drawing", async (ClaimsPrincipal user, CancellationToken cancellation) =>
{
    var subject = user.FindFirstValue("sub");
    var tenant = user.FindFirstValue("tenant_id");
    var role = user.FindFirstValue("role")?.ToLowerInvariant();
    var boundTenant = Environment.GetEnvironmentVariable("CAD360_LOCAL_TENANT_ID");
    if (string.IsNullOrWhiteSpace(subject) || string.IsNullOrWhiteSpace(tenant)
        || string.IsNullOrWhiteSpace(boundTenant) || !string.Equals(tenant, boundTenant, StringComparison.Ordinal)
        || role is not ("viewer" or "editor" or "approver")) return Results.Forbid();
    var cli = Environment.GetEnvironmentVariable("CAD360_MCP_CLI_PATH");
    if (string.IsNullOrWhiteSpace(cli) || !Path.IsPathFullyQualified(cli) || !File.Exists(cli))
        return Results.Problem("Trusted CAD CLI path is not configured.", statusCode:503);
    var address = Environment.GetEnvironmentVariable("CAD360_MCP_CONNECT_ADDRESS") ?? "127.0.0.1:7979";
    if (!address.StartsWith("127.0.0.1:", StringComparison.Ordinal)
        || !int.TryParse(address.AsSpan("127.0.0.1:".Length), out var port) || port is < 1 or > 65535)
        return Results.Problem("CAD connection must be bound to IPv4 loopback.", statusCode:503);
    using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellation);
    timeout.CancelAfter(TimeSpan.FromSeconds(10));
    using var process = new Process();
    process.StartInfo = new ProcessStartInfo
    {
        FileName = cli,
        UseShellExecute = false,
        RedirectStandardInput = true,
        RedirectStandardOutput = true,
        RedirectStandardError = false,
        CreateNoWindow = true
    };
    process.StartInfo.ArgumentList.Add("mcp");
    process.StartInfo.ArgumentList.Add("--connect");
    process.StartInfo.ArgumentList.Add(address);
    process.StartInfo.Environment["CAD360_MCP_ACCESS_MODE"] = "readonly";
    try
    {
        if (!process.Start()) return Results.Problem("Unable to launch CAD MCP.", statusCode:503);
        var init = JsonSerializer.Serialize(new {jsonrpc="2.0",id=1,method="initialize",
            @params=new {protocolVersion="2025-06-18",capabilities=new {},clientInfo=new {name="cad360-oidc-readonly",version="1"}}});
        await process.StandardInput.WriteLineAsync(init.AsMemory(), timeout.Token);
        await process.StandardInput.FlushAsync(timeout.Token);
        var initialized = await process.StandardOutput.ReadLineAsync(timeout.Token);
        if (string.IsNullOrWhiteSpace(initialized))
            return Results.Problem("MCP initialize failed.",statusCode:502);
        using (var handshake = JsonDocument.Parse(initialized))
        {
            if (!handshake.RootElement.TryGetProperty("result",out _))
                return Results.Problem("MCP rejected initialization.",statusCode:502);
        }
        const string read = "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"inspect_drawing\",\"arguments\":{\"entities\":false}}}";
        await process.StandardInput.WriteLineAsync(read.AsMemory(),timeout.Token);
        await process.StandardInput.FlushAsync(timeout.Token);
        var reply = await process.StandardOutput.ReadLineAsync(timeout.Token);
        if (string.IsNullOrWhiteSpace(reply)) return Results.Problem("CAD MCP returned no response.",statusCode:502);
        using var result = JsonDocument.Parse(reply);
        if (result.RootElement.TryGetProperty("error",out _)) return Results.Problem("CAD MCP denied read.",statusCode:502);
        if (!result.RootElement.TryGetProperty("result",out var payload))
            return Results.Problem("Invalid MCP response.",statusCode:502);
        return Results.Json(new {scopedTenant=tenant,subject,tool="inspect_drawing",readOnly=true,
            mcpResult=payload.Clone()});
    }
    catch (OperationCanceledException)
    {
        return Results.Problem("CAD MCP read timed out.",statusCode:504);
    }
    catch (Exception ex) when (ex is System.ComponentModel.Win32Exception or IOException or JsonException or InvalidOperationException)
    {
        // Deliberately omit exception details (could contain private machine paths).
        return Results.Problem("CAD MCP connection failed.",statusCode:502);
    }
    finally
    {
        if (!process.HasExited) process.Kill(entireProcessTree:true);
    }
}).RequireAuthorization("CAD360Scoped");

app.Run();
