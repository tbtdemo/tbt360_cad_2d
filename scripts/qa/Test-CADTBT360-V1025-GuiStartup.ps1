param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$EvidenceDirectory,
    [int]$WaitSeconds = 35
)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Path $EvidenceDirectory -Force | Out-Null
$exe = (Resolve-Path -LiteralPath $Executable).Path
$report = Join-Path $EvidenceDirectory 'gui-startup-result.json'
$stdout = Join-Path $EvidenceDirectory 'gui-stdout.log'
$stderr = Join-Path $EvidenceDirectory 'gui-stderr.log'
$p = $null
$started = [DateTimeOffset]::UtcNow
$status = 'NOT_VERIFIED'
$detail = ''
$windowTitle = ''
$windowHandle = '0'
$exitCode = $null
try {
    $p = Start-Process -FilePath $exe -WorkingDirectory (Split-Path $exe -Parent) -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru
    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 1000
        $p.Refresh()
        if ($p.HasExited) { $exitCode = $p.ExitCode; throw "GUI exited before a window appeared (ExitCode=$exitCode)" }
        if ($p.MainWindowHandle -ne [IntPtr]::Zero) {
            $windowHandle = [string]$p.MainWindowHandle.ToInt64()
            $windowTitle = $p.MainWindowTitle
            $status = 'PROCESS_AND_TOP_LEVEL_WINDOW_OBSERVED'
            break
        }
    }
    if ($status -ne 'PROCESS_AND_TOP_LEVEL_WINDOW_OBSERVED') {
        throw "No visible top-level GUI window within $WaitSeconds seconds. This runner may be noninteractive, or GUI initialization failed."
    }
    # Observing the window is startup evidence only; it is not manual GUI acceptance.
    Start-Sleep -Seconds 2
    $p.Refresh()
    if ($p.HasExited) { $exitCode = $p.ExitCode; throw "GUI exited immediately after initial window creation (ExitCode=$exitCode)" }
    $detail = 'GUI process survived startup and exposed a top-level window on this Windows runner. Manual visual/functional acceptance remains NOT_VERIFIED.'
} catch {
    $status = 'FAIL_OR_INCONCLUSIVE'
    $detail = $_.Exception.Message
    throw
} finally {
    if ($null -ne $p) {
        try {
            $p.Refresh()
            if (-not $p.HasExited) {
                Stop-Process -Id $p.Id -Force -ErrorAction Stop
                $p.WaitForExit(10000) | Out-Null
            } else { $exitCode = $p.ExitCode }
        } catch { Write-Warning "Could not cleanly terminate GUI PID $($p.Id): $_" }
    }
    $record = [ordered]@{
        utcStarted = $started.ToString('o')
        utcFinished = [DateTimeOffset]::UtcNow.ToString('o')
        executable = $exe
        exeSha256 = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
        runId = $env:GITHUB_RUN_ID
        sourceCommit = $env:GITHUB_SHA
        osVersion = [Environment]::OSVersion.VersionString
        sessionName = $env:SESSIONNAME
        status = $status
        detail = $detail
        mainWindowHandle = $windowHandle
        mainWindowTitle = $windowTitle
        exitCode = $exitCode
        manualGuiAcceptance = 'NOT_VERIFIED'
        independentAutocadInterop = 'NOT_VERIFIED'
    }
    $record | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $report -Encoding utf8
    Write-Host ($record | ConvertTo-Json -Depth 5)
}
