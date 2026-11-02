using System.Security.Claims;
using System.Text.Json;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.IdentityModel.Tokens;
using Npgsql;
using NpgsqlTypes;

var builder = WebApplication.CreateBuilder(args);
var issuer = builder.Configuration["Auth:Authority"] ?? throw new InvalidOperationException("Auth:Authority is required");
var audience = builder.Configuration["Auth:Audience"] ?? throw new InvalidOperationException("Auth:Audience is required");
var connectionString = builder.Configuration.GetConnectionString("Cad360") ?? throw new InvalidOperationException("ConnectionStrings:Cad360 is required");
if (!Uri.TryCreate(issuer, UriKind.Absolute, out var authorityUri) || authorityUri.Scheme != Uri.UriSchemeHttps) throw new InvalidOperationException("Auth authority must use HTTPS");
builder.Services.AddAuthentication(JwtBearerDefaults.AuthenticationScheme).AddJwtBearer(options =>
{
    options.Authority = issuer;
    options.Audience = audience;
    options.RequireHttpsMetadata = true;
    options.MapInboundClaims = false;
    options.TokenValidationParameters = new TokenValidationParameters { ValidateIssuer = true, ValidateAudience = true, ValidateLifetime = true, ValidateIssuerSigningKey = true, NameClaimType = "sub" };
});
builder.Services.AddAuthorization();
builder.Services.AddSingleton(NpgsqlDataSource.Create(connectionString));
var app = builder.Build();
app.UseAuthentication();
app.UseAuthorization();

static Guid AuthenticatedAccount(ClaimsPrincipal principal)
{
    var sub = principal.FindFirstValue("sub");
    if (!Guid.TryParse(sub, out var account) || account == Guid.Empty) throw new UnauthorizedAccessException("Invalid authenticated subject");
    return account;
}
static IResult Fail(Exception error) => error switch
{
    UnauthorizedAccessException => Results.Unauthorized(),
    PostgresException { SqlState: "42501" } => Results.StatusCode(403),
    PostgresException { SqlState: "P0002" } => Results.NotFound(),
    PostgresException { SqlState: "23505" or "40001" } => Results.Conflict(new { error = "conflict" }),
    PostgresException { SqlState: "55P03" } => Results.StatusCode(423),
    PostgresException { SqlState: "22023" } => Results.BadRequest(new { error = "invalid request" }),
    _ => Results.Problem(statusCode: 500, title: "Storage operation failed")
};
static async Task<IResult> ExecuteWrite(NpgsqlDataSource source, ClaimsPrincipal principal, Guid project, Guid obj, string sql, Action<NpgsqlCommand> bind, CancellationToken ct)
{
    try
    {
        var actor = AuthenticatedAccount(principal);
        if (project == Guid.Empty || obj == Guid.Empty) return Results.BadRequest();

        await using var conn = await source.OpenConnectionAsync(ct);
        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("project", project);
        cmd.Parameters.AddWithValue("object", obj);
        cmd.Parameters.AddWithValue("actor", actor);
        bind(cmd);
        var result = await cmd.ExecuteScalarAsync(ct);
        return Results.Ok(new { revision = Convert.ToInt64(result) });
    }
    catch (Exception ex) when (ex is PostgresException or UnauthorizedAccessException) { return Fail(ex); }
}
app.MapPost("/api/bim/projects/{project:guid}/objects", async (Guid project, CreateObject body, NpgsqlDataSource db, ClaimsPrincipal principal, CancellationToken ct) =>
    body.Geometry.ValueKind != JsonValueKind.Object || body.FloorId == Guid.Empty || string.IsNullOrWhiteSpace(body.Discipline) || string.IsNullOrWhiteSpace(body.SourceHandle) || string.IsNullOrWhiteSpace(body.EntityType) ? Results.BadRequest(new { error = "invalid geometry or incomplete object scope" }) : await ExecuteWrite(db, principal, project, body.ObjectId,
        "SELECT bim_create_design_object(@project,@object,@actor,@floor,@discipline,@handle,@kind,@payload)", cmd =>
        {
            cmd.Parameters.AddWithValue("floor", body.FloorId);
            cmd.Parameters.AddWithValue("discipline", body.Discipline);
            cmd.Parameters.AddWithValue("handle", body.SourceHandle);
            cmd.Parameters.AddWithValue("kind", body.EntityType);
            cmd.Parameters.Add("payload", NpgsqlDbType.Jsonb).Value = body.Geometry.GetRawText();
        }, ct)).RequireAuthorization();
app.MapPut("/api/bim/projects/{project:guid}/objects/{obj:guid}", async (Guid project, Guid obj, EditObject body, NpgsqlDataSource db, ClaimsPrincipal principal, CancellationToken ct) =>
    body.Geometry.ValueKind != JsonValueKind.Object || body.ExpectedRevision < 1 ? Results.BadRequest(new { error = "invalid geometry or expected revision" }) : await ExecuteWrite(db, principal, project, obj,
        "SELECT bim_update_design_object(@project,@object,@actor,@revision,@payload)", cmd =>
        {
            cmd.Parameters.AddWithValue("revision", body.ExpectedRevision);
            cmd.Parameters.Add("payload", NpgsqlDbType.Jsonb).Value = body.Geometry.GetRawText();
        }, ct)).RequireAuthorization();
app.MapDelete("/api/bim/projects/{project:guid}/objects/{obj:guid}", async (Guid project, Guid obj, long expectedRevision, NpgsqlDataSource db, ClaimsPrincipal principal, CancellationToken ct) =>
    expectedRevision < 1 ? Results.BadRequest(new { error = "invalid expected revision" }) : await ExecuteWrite(db, principal, project, obj,
        "SELECT bim_delete_design_object(@project,@object,@actor,@revision)", cmd => cmd.Parameters.AddWithValue("revision", expectedRevision), ct)).RequireAuthorization();
app.MapGet("/api/bim/projects/{project:guid}/changes", async (Guid project, long afterRevision, int? limit, ClaimsPrincipal principal, NpgsqlDataSource db, CancellationToken ct) =>
{
    try
    {
        var actor = AuthenticatedAccount(principal);
        if (project == Guid.Empty || afterRevision < 0 || limit is < 1 or > 1000) return Results.BadRequest(new { error = "invalid change feed cursor or limit" });
        await using var connection = await db.OpenConnectionAsync(ct);
        await using var command = new NpgsqlCommand("SELECT project_revision,design_object_id,source_handle,operation,object_revision,actor_account_id,occurred_at FROM bim_pull_project_changes(@project,@actor,@cursor,@limit)", connection);
        command.Parameters.AddWithValue("project", project);
        command.Parameters.AddWithValue("actor", actor);
        command.Parameters.AddWithValue("cursor", afterRevision);
        command.Parameters.AddWithValue("limit", limit ?? 500);
        await using var reader = await command.ExecuteReaderAsync(ct);
        var items = new List<object>();
        while (await reader.ReadAsync(ct)) items.Add(new { projectRevision = reader.GetInt64(0), objectId = reader.GetGuid(1), sourceHandle = reader.GetString(2), operation = reader.GetString(3), objectRevision = reader.GetInt64(4), actorAccountId = reader.GetGuid(5), occurredAt = reader.GetDateTime(6) });
        return Results.Ok(items);
    }
    catch (Exception ex) when (ex is PostgresException or UnauthorizedAccessException) { return Fail(ex); }
}).RequireAuthorization();

// Batched writes are serialized by project row. Each failure rolls back all edits,
// audit, change feed and the idempotency receipt in the same transaction.
app.MapPost("/api/bim/projects/{project:guid}/changesets", async (Guid project, ChangeBatch body, ClaimsPrincipal principal, NpgsqlDataSource db, CancellationToken ct) =>
{
    try
    {
        var actor = AuthenticatedAccount(principal);
        if (project == Guid.Empty || body.BatchId == Guid.Empty || body.Items is null || body.Items.Count is < 1 or > 100) return Results.BadRequest();
        if (body.BaseProjectRevision is < 0) return Results.BadRequest(new { error = "invalid base project revision" });
        if (body.Items.Any(item => item.ObjectId == Guid.Empty || item.Operation is null || !new[] { "CREATE", "UPDATE", "DELETE" }.Contains(item.Operation.ToUpperInvariant()))) return Results.BadRequest();
        if (body.Items.Select(item => item.ObjectId).Distinct().Count() != body.Items.Count) return Results.BadRequest(new { error = "duplicate object id in batch" });
        if (body.Items.Any(item => item.Operation!.ToUpperInvariant() != "CREATE" && item.ExpectedRevision < 1)) return Results.BadRequest(new { error = "invalid expected revision" });
        if (body.Items.Any(item => item.Operation!.ToUpperInvariant() != "DELETE" && item.Geometry.ValueKind != JsonValueKind.Object)) return Results.BadRequest(new { error = "geometry must be JSON object" });
        if (body.Items.Any(item => item.Operation!.ToUpperInvariant() == "CREATE" && (item.FloorId == Guid.Empty || string.IsNullOrWhiteSpace(item.Discipline) || string.IsNullOrWhiteSpace(item.SourceHandle) || string.IsNullOrWhiteSpace(item.EntityType)))) return Results.BadRequest(new { error = "incomplete create scope" });
        var canonical = JsonSerializer.Serialize(new { body.BaseProjectRevision, body.Items });
        var hash = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(canonical)));
        await using var conn = await db.OpenConnectionAsync(ct);
        await using var tx = await conn.BeginTransactionAsync(ct);
        long currentProjectRevision;
        await using (var lockCmd = new NpgsqlCommand("SELECT revision FROM bim_projects WHERE id=@project FOR UPDATE", conn, tx))
        {
            lockCmd.Parameters.AddWithValue("project", project);
            var lockedRevision = await lockCmd.ExecuteScalarAsync(ct);
            if (lockedRevision is null) return Results.NotFound();
            currentProjectRevision = Convert.ToInt64(lockedRevision);
        }
        // Replays must not bypass current project membership after access is revoked.
        await using (var access = new NpgsqlCommand("SELECT EXISTS(SELECT 1 FROM bim_project_members WHERE project_id=@project AND account_id=@actor)", conn, tx))
        {
            access.Parameters.AddWithValue("project", project);
            access.Parameters.AddWithValue("actor", actor);
            if (await access.ExecuteScalarAsync(ct) is not true) return Results.Forbid();
        }
        await using (var prior = new NpgsqlCommand("SELECT request_sha256,response_json::text FROM bim_change_batch_receipts WHERE project_id=@project AND account_id=@actor AND client_batch_id=@batch", conn, tx))
        {
            prior.Parameters.AddWithValue("project", project);
            prior.Parameters.AddWithValue("actor", actor);
            prior.Parameters.AddWithValue("batch", body.BatchId);
            await using var reader = await prior.ExecuteReaderAsync(ct);
            if (await reader.ReadAsync(ct))
            {
                var matched = reader.GetString(0) == hash;
                var saved = reader.GetString(1);
                await reader.CloseAsync();
                return matched ? Results.Content(saved, "application/json") : Results.Conflict(new { error = "batch id reused for different payload" });
            }
        }
        if (body.BaseProjectRevision is long baseRevision && currentProjectRevision != baseRevision) return Results.Conflict(new { error = "stale project revision", currentProjectRevision });
        var accepted = new List<object>();
        foreach (var item in body.Items)
        {
            var operation = item.Operation?.ToUpperInvariant();
            string sql = operation switch
            {
                "CREATE" => "SELECT bim_create_design_object(@project,@object,@actor,@floor,@discipline,@handle,@kind,@payload)",
                "UPDATE" => "SELECT bim_update_design_object(@project,@object,@actor,@revision,@payload)",
                "DELETE" => "SELECT bim_delete_design_object(@project,@object,@actor,@revision)",
                _ => throw new ArgumentException("Unsupported operation")
            };
            await using var command = new NpgsqlCommand(sql, conn, tx);
            command.Parameters.AddWithValue("project", project);
            command.Parameters.AddWithValue("object", item.ObjectId);
            command.Parameters.AddWithValue("actor", actor);
            if (operation == "CREATE")
            {
                command.Parameters.AddWithValue("floor", item.FloorId);
                command.Parameters.AddWithValue("discipline", item.Discipline ?? "");
                command.Parameters.AddWithValue("handle", item.SourceHandle ?? "");
                command.Parameters.AddWithValue("kind", item.EntityType ?? "");
            }
            else command.Parameters.AddWithValue("revision", item.ExpectedRevision);
            if (operation != "DELETE") command.Parameters.Add("payload", NpgsqlDbType.Jsonb).Value = item.Geometry.GetRawText();
            accepted.Add(new { item.ObjectId, operation, revision = Convert.ToInt64(await command.ExecuteScalarAsync(ct)) });
        }
        var response = JsonSerializer.Serialize(new { batchId = body.BatchId, accepted });
        await using (var receipt = new NpgsqlCommand("INSERT INTO bim_change_batch_receipts(project_id,account_id,client_batch_id,request_sha256,response_json) VALUES(@project,@actor,@batch,@hash,@response)", conn, tx))
        {
            receipt.Parameters.AddWithValue("project", project);
            receipt.Parameters.AddWithValue("actor", actor);
            receipt.Parameters.AddWithValue("batch", body.BatchId);
            receipt.Parameters.AddWithValue("hash", hash);
            receipt.Parameters.Add("response", NpgsqlDbType.Jsonb).Value = response;
            await receipt.ExecuteNonQueryAsync(ct);
        }
        await tx.CommitAsync(ct);
        return Results.Content(response, "application/json");
    }
    catch (ArgumentException) { return Results.BadRequest(); }
    catch (Exception ex) when (ex is PostgresException or UnauthorizedAccessException) { return Fail(ex); }
}).RequireAuthorization();

// Initial sync and reconnect handshake. Only current project members may read the cursor.
app.MapGet("/api/bim/projects/{project:guid}/sync-state", async (Guid project, ClaimsPrincipal principal, NpgsqlDataSource db, CancellationToken ct) =>
{
    try
    {
        var actor = AuthenticatedAccount(principal);
        await using var conn = await db.OpenConnectionAsync(ct);
        await using var cmd = new NpgsqlCommand("SELECT p.revision FROM bim_projects p WHERE p.id=@project AND EXISTS(SELECT 1 FROM bim_project_members m WHERE m.project_id=p.id AND m.account_id=@actor)", conn);
        cmd.Parameters.AddWithValue("project", project);
        cmd.Parameters.AddWithValue("actor", actor);
        var revision = await cmd.ExecuteScalarAsync(ct);
        return revision is null ? Results.NotFound() : Results.Ok(new { projectId = project, projectRevision = Convert.ToInt64(revision) });
    }
    catch (Exception ex) when (ex is PostgresException or UnauthorizedAccessException) { return Fail(ex); }
}).RequireAuthorization();

app.Run();


public record CreateObject(Guid ObjectId, Guid FloorId, string Discipline, string SourceHandle, string EntityType, JsonElement Geometry);
public record EditObject(long ExpectedRevision, JsonElement Geometry);

public record ChangeBatch(Guid BatchId, List<BatchItem> Items, long? BaseProjectRevision = null);
public record BatchItem(string? Operation, Guid ObjectId, Guid FloorId, string? Discipline, string? SourceHandle, string? EntityType, long ExpectedRevision, JsonElement Geometry);
