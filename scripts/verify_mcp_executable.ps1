# CAD 360 Windows release smoke: verify the actual installed/Portable CLI
# can start a JSON-RPC MCP session and exposes the PNG-to-CAD tool.
# Usage: pwsh -NoProfile -File scripts/verify_mcp_executable.ps1 -Exe "path/CADTBT360_CLI.exe"
param([Parameter(Mandatory=$true)][string]$Exe)
$ErrorActionPreference = 'Stop'
$exePath = (Resolve-Path -LiteralPath $Exe).Path
$psi = [System.Diagnostics.ProcessStartInfo]::new()
$psi.FileName = $exePath
$psi.Arguments = 'mcp'
$psi.UseShellExecute = $false
$psi.CreateNoWindow = $true
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$process = [System.Diagnostics.Process]::Start($psi)
if ($null -eq $process) { throw "Cannot launch installed MCP CLI: $exePath" }
try {
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $requests = @(
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"cad360-release-smoke","version":"1"}}}',
        '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}',
        '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}',
        '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"raster_to_cad_lines","arguments":{"path":"__CAD360_NONEXISTENT_TEST_IMAGE__.png","preview":true}}}'
    )
    foreach ($request in $requests) { $process.StandardInput.WriteLine($request) }
    $process.StandardInput.Close()
    if (-not $process.WaitForExit(20000)) {
        $process.Kill($true)
        throw "MCP CLI timed out after 20 seconds: $exePath"
    }
    $stdout = $stdoutTask.GetAwaiter().GetResult()
    $stderr = $stderrTask.GetAwaiter().GetResult()
    if ($process.ExitCode -ne 0) { throw "MCP CLI returned exit code $($process.ExitCode): $stderr" }
    $replies = @($stdout -split '\r?\n' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object { $_ | ConvertFrom-Json })
    $initialized = @($replies | Where-Object { $_.id -eq 1 -and $_.result.protocolVersion -eq '2025-06-18' })
    if ($initialized.Count -ne 1) { throw 'Installed MCP initialize handshake failed' }
    $tools = @($replies | Where-Object { $_.id -eq 2 })
    if ($tools.Count -ne 1) { throw 'Installed MCP tools/list did not return one response' }
    $imageTool = @($tools[0].result.tools | Where-Object { $_.name -eq 'raster_to_cad_lines' })
    if ($imageTool.Count -ne 1) { throw 'Installed MCP CLI does not expose raster_to_cad_lines' }
    $errors = @($replies | Where-Object { $_.id -eq 3 })
    if ($errors.Count -ne 1 -or $errors[0].result.isError -ne $true) {
        throw 'Installed MCP image tool did not safely reject nonexistent image source'
    }
    Write-Host "MCP smoke PASS: handshake, raster_to_cad_lines discovery and safe source rejection ($exePath)"
} finally {
    if (-not $process.HasExited) { $process.Kill($true) }
    $process.Dispose()
}
