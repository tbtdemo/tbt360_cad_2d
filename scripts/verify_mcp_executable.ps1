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
# Self-contained, reproducible PNG fixture (64x32 white, one long black stroke).
# It is created in a temporary location; neither the app nor a network is needed.
$fixtureId = [guid]::NewGuid().ToString("N")
$png = Join-Path ([System.IO.Path]::GetTempPath()) ("cad360-mcp-smoke-" + $fixtureId + ".png")
$dxf = Join-Path ([System.IO.Path]::GetTempPath()) ("cad360-mcp-smoke-" + $fixtureId + ".dxf")
$dwg = Join-Path ([System.IO.Path]::GetTempPath()) ("cad360-mcp-smoke-" + $fixtureId + ".dwg")
$pngBase64 = 'iVBORw0KGgoAAAANSUhEUgAAAEAAAAAgCAIAAAAt/+nTAAAAUUlEQVR4nO3WMQoAIAwEQU/8/5djL4iFxRLY6dJlu0tVjc4m/cAvA2gG0AygGUAzgLaOOwnyx9Nts8UxBzOAZgDNAJoBNANoBtAMoBlAax+wAU7iCTuV73xMAAAAAElFTkSuQmCC'
[System.IO.File]::WriteAllBytes($png, [Convert]::FromBase64String($pngBase64))
$process = [System.Diagnostics.Process]::Start($psi)
if ($null -eq $process) { Remove-Item -LiteralPath $png -ErrorAction SilentlyContinue; throw "Cannot launch installed MCP CLI: $exePath" }
try {
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $requests = @(
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"cad360-release-smoke","version":"1"}}}',
        '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}',
        '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}',
        '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"raster_to_cad_lines","arguments":{"path":"__CAD360_NONEXISTENT_TEST_IMAGE__.png","preview":true}}}'
    )
    $previewArgs = @{ path=$png; preview=$true; minLengthPixels=16; includeDiagonals=$false }
    $commitArgs = @{ path=$png; preview=$false; minLengthPixels=16; includeDiagonals=$false }
    $requests += @(
        (@{ jsonrpc='2.0'; id=4; method='tools/call'; params=@{name='raster_to_cad_lines';arguments=$previewArgs}} | ConvertTo-Json -Depth 8 -Compress),
        (@{ jsonrpc='2.0'; id=5; method='tools/call'; params=@{name='raster_to_cad_lines';arguments=$commitArgs}} | ConvertTo-Json -Depth 8 -Compress),
        (@{ jsonrpc='2.0'; id=6; method='tools/call'; params=@{name='query_entities';arguments=@{type='Line'}}} | ConvertTo-Json -Depth 8 -Compress),
        (@{ jsonrpc='2.0'; id=7; method='tools/call'; params=@{name='execute';arguments=@{command='undo';params=@{count=1}}}} | ConvertTo-Json -Depth 8 -Compress),
        (@{ jsonrpc='2.0'; id=8; method='tools/call'; params=@{name='query_entities';arguments=@{type='Line'}}} | ConvertTo-Json -Depth 8 -Compress),
        (@{ jsonrpc='2.0'; id=9; method='tools/call'; params=@{name='raster_to_cad_lines';arguments=$commitArgs}} | ConvertTo-Json -Depth 8 -Compress),
        (@{ jsonrpc='2.0'; id=10; method='tools/call'; params=@{name='save';arguments=@{path=$dxf}}} | ConvertTo-Json -Depth 8 -Compress)
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
    function RequireToolReply([int]$id) {
        $match = @($replies | Where-Object { $_.id -eq $id })
        if ($match.Count -ne 1 -or $match[0].result.isError -eq $true -or $match[0].error) {
            throw "Installed MCP tool call $id failed or returned a protocol error: $($match | ConvertTo-Json -Depth 5 -Compress)"
        }
        if ($match[0].result.content.Count -lt 1 -or $match[0].result.content[0].type -ne 'text') {
            throw "MCP tool call $id returned no JSON text"
        }
        return ($match[0].result.content[0].text | ConvertFrom-Json)
    }
    $preview = RequireToolReply 4
    if ($preview.preview -ne $true -or $preview.strokes -ne 1 -or $preview.editableEntitiesCreated -ne 0) {
        throw "Installed MCP real-PNG preview mismatch: $($preview | ConvertTo-Json -Compress)"
    }
    $commit = RequireToolReply 5
    if ($commit.editableEntitiesCreated -ne 1 -or $commit.strokes -ne 1) {
        throw "Installed MCP real-PNG-to-LINE commit mismatch: $($commit | ConvertTo-Json -Compress)"
    }
    $entitiesBeforeUndo = RequireToolReply 6
    if ($entitiesBeforeUndo.count -ne 1) { throw 'Installed MCP did not create an editable LINE from the real PNG' }
    $undoResult = RequireToolReply 7
    $entitiesAfterUndo = RequireToolReply 8
    if ($entitiesAfterUndo.count -ne 0) { throw 'Installed MCP failed to undo its raster-to-CAD batch' }
    $repeatedImport = RequireToolReply 9
    if ($repeatedImport.editableEntitiesCreated -ne 1) { throw 'Installed MCP could not recreate LINE after Undo' }
    $saved = RequireToolReply 10
    if (!(Test-Path -LiteralPath $dxf)) { throw "Installed MCP did not save AI-created LINE to DXF: $dxf" }
    & $exePath convert $dxf $dwg | Out-Host
    if ($LASTEXITCODE -ne 0 -or !(Test-Path -LiteralPath $dwg)) { throw 'Installed CLI could not convert AI-generated DXF into DWG' }
    $bytes = [System.IO.File]::ReadAllBytes($dwg)
    if ($bytes.Length -lt 64 -or [System.Text.Encoding]::ASCII.GetString($bytes, 0, 6) -ne 'AC1032') {
        throw 'AI-generated DWG does not contain the expected R2018 header'
    }
    $dxfInfoText = & $exePath info $dxf
    if ($LASTEXITCODE -ne 0) { throw 'Installed CLI cannot reopen AI-generated DXF' }
    $dwgInfoText = & $exePath info $dwg
    if ($LASTEXITCODE -ne 0) { throw 'Installed CLI cannot reopen AI-generated DWG' }
    $dxfInfo = ($dxfInfoText -join [char]10) | ConvertFrom-Json
    $dwgInfo = ($dwgInfoText -join [char]10) | ConvertFrom-Json
    if ($dxfInfo.entityCount -ne 1 -or $dwgInfo.entityCount -ne 1) {
        throw "AI-created LINE was lost during DXF/DWG roundtrip: DXF=$($dxfInfo.entityCount), DWG=$($dwgInfo.entityCount)"
    }
    Write-Host "MCP smoke PASS: real PNG preview, editable LINE, Undo, AI DXF/DWG save-reopen ($exePath)"
} finally {
    if (-not $process.HasExited) { $process.Kill($true) }
    $process.Dispose()
    Remove-Item -LiteralPath @($png,$dxf,$dwg) -Force -ErrorAction SilentlyContinue
}
