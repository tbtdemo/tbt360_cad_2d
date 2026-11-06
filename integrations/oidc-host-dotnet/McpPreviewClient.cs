using System.ComponentModel;
using System.Diagnostics;
using System.Text.Json;

// This class ONLY invokes CAD AI preview, never confirms or executes a plan.
// It trusts no request-supplied tool name, command, drawing ID or TCP endpoint.
public static class McpPreviewClient
{
    public static async Task<JsonElement> PreviewAsync(string text,CancellationToken cancellation)
    {
        if (!HostGrantPolicy.ValidPreviewText(text)) throw new ArgumentException("Invalid AI preview text");
        var cli=Environment.GetEnvironmentVariable("CAD360_MCP_CLI_PATH");
        if (string.IsNullOrWhiteSpace(cli)||!Path.IsPathFullyQualified(cli)||!File.Exists(cli))
            throw new InvalidOperationException("CAD CLI not configured");
        var address=Environment.GetEnvironmentVariable("CAD360_MCP_CONNECT_ADDRESS")??"127.0.0.1:7979";
        if (!HostGrantPolicy.IsLoopbackAddress(address))
            throw new InvalidOperationException("Non-loopback CAD control endpoint denied");
        using var timeout=CancellationTokenSource.CreateLinkedTokenSource(cancellation);
        timeout.CancelAfter(TimeSpan.FromSeconds(10));
        using var process=new Process();
        process.StartInfo=new ProcessStartInfo {
            FileName=cli,UseShellExecute=false,RedirectStandardInput=true,
            RedirectStandardOutput=true,RedirectStandardError=false,CreateNoWindow=true
        };
        process.StartInfo.ArgumentList.Add("mcp");
        process.StartInfo.ArgumentList.Add("--connect");
        process.StartInfo.ArgumentList.Add(address);
        // The reviewed profile permits preview but not arbitrary execute/script.
        process.StartInfo.Environment["CAD360_MCP_ACCESS_MODE"]="reviewed";
        var started=false;
        try {
            if (!process.Start()) throw new InvalidOperationException("MCP process failed to start");
            started=true;
            var init=JsonSerializer.Serialize(new {jsonrpc="2.0",id=1,method="initialize",
                @params=new {protocolVersion="2025-06-18",capabilities=new {},clientInfo=new {name="cad360-oidc-preview",version="1"}}});
            await process.StandardInput.WriteLineAsync(init.AsMemory(),timeout.Token);
            await process.StandardInput.FlushAsync(timeout.Token);
            var hello=await process.StandardOutput.ReadLineAsync(timeout.Token);
            if (hello is null) throw new IOException("MCP handshake missing");
            using (var handshake=JsonDocument.Parse(hello)) {
                if (!handshake.RootElement.TryGetProperty("result",out _))
                    throw new InvalidOperationException("MCP handshake rejected");
            }
            await process.StandardInput.WriteLineAsync("""{"jsonrpc":"2.0","method":"notifications/initialized"}""".AsMemory(),timeout.Token);
            // JSON serializer prevents injection of tool or preview/confirm flags.
            var command=JsonSerializer.Serialize(new {jsonrpc="2.0",id=2,method="tools/call",
                @params=new {name="ai_draw_plan_vietnamese",
                    arguments=new {text,preview=true,confirm=false,createDoorBlock=false,applyLayers=false}}});
            await process.StandardInput.WriteLineAsync(command.AsMemory(),timeout.Token);
            await process.StandardInput.FlushAsync(timeout.Token);
            var answer=await process.StandardOutput.ReadLineAsync(timeout.Token);
            if (answer is null) throw new IOException("MCP preview response missing");
            using var doc=JsonDocument.Parse(answer);
            if (!doc.RootElement.TryGetProperty("result",out var result))
                throw new InvalidOperationException("CAD AI preview rejected");
            return result.Clone();
        }
        finally {
            if (started) {
                try {if (!process.HasExited) process.Kill(entireProcessTree:true);}
                catch (InvalidOperationException) { /* already stopped */ }
            }
        }
    }
}
