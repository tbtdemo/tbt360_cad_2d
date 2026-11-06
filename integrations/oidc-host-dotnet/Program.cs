// CAD 360 verified OIDC host sample. This endpoint validates signatures using
// ASP.NET Core JwtBearer + authority JWKS. It does NOT proxy MCP commands yet:
// a trusted, authenticated in-process bridge is required before granting CAD access.
using System.Security.Claims;
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
app.Run();
