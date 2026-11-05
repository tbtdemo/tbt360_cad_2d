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
$processId = $null
$sessionId = $null
$failureKind = 'NONE'
$screenshotStatus = 'NOT_ATTEMPTED'
$windowRect = $null
try {
    $p = Start-Process -FilePath $exe -WorkingDirectory (Split-Path $exe -Parent) -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru
    $processId = $p.Id
    $sessionId = $p.SessionId
    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 1000
        $p.Refresh()
        if ($p.HasExited) { $exitCode = $p.ExitCode; $failureKind = "PROCESS_EXITED_BEFORE_WINDOW"; throw "GUI exited before a window appeared (ExitCode=$exitCode)" }
        if ($p.MainWindowHandle -ne [IntPtr]::Zero) {
            $windowHandle = [string]$p.MainWindowHandle.ToInt64()
            $windowTitle = $p.MainWindowTitle
            $status = 'PROCESS_AND_TOP_LEVEL_WINDOW_OBSERVED'
            break
        }
    }
    if ($status -ne 'PROCESS_AND_TOP_LEVEL_WINDOW_OBSERVED') {
        $failureKind = "WINDOW_NOT_OBSERVED"
        throw "No top-level GUI window observed within $WaitSeconds seconds. Cannot distinguish noninteractive runner from GUI initialization failure."
    }
    # Observing the window is startup evidence only; it is not manual GUI acceptance.
    Start-Sleep -Seconds 2
    $p.Refresh()
    if ($p.HasExited) { $exitCode = $p.ExitCode; $failureKind = "PROCESS_EXITED_AFTER_WINDOW"; throw "GUI exited immediately after initial window creation (ExitCode=$exitCode)" }
    # Capture evidence from this exact running executable, not a substituted CAD screenshot.
    # Screen capture may be blocked by CI runner desktop/session restrictions; do not
    # turn a healthy GUI process into FAIL solely because capturing pixels is unavailable.
    try {
        Add-Type -AssemblyName System.Drawing
        if (-not ('Cad360GuiEvidence' -as [type])) {
            Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Cad360GuiEvidence {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
}
'@
        }
        $bounds = New-Object Cad360GuiEvidence+RECT
        if (-not [Cad360GuiEvidence]::GetWindowRect($p.MainWindowHandle, [ref]$bounds)) { throw 'GetWindowRect failed' }
        $width = $bounds.Right - $bounds.Left
        $height = $bounds.Bottom - $bounds.Top
        $windowRect = [ordered]@{ left=$bounds.Left; top=$bounds.Top; width=$width; height=$height }
        if ($width -le 1 -or $height -le 1 -or $width -gt 16384 -or $height -gt 16384) { throw "Invalid GUI window bounds: ${width}x${height}" }
        $bitmap = New-Object System.Drawing.Bitmap($width, $height)
        try {
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $graphics.CopyFromScreen($bounds.Left, $bounds.Top, 0, 0, $bitmap.Size)
            } finally { $graphics.Dispose() }
            $bitmap.Save((Join-Path $EvidenceDirectory 'gui-window-screenshot.png'), [System.Drawing.Imaging.ImageFormat]::Png)
            $screenshotStatus = 'CAPTURED'
        } finally { $bitmap.Dispose() }
    } catch {
        $screenshotStatus = "UNAVAILABLE: $($_.Exception.Message)"
        Write-Warning "GUI screenshot could not be captured in this Windows session: $($_.Exception.Message)"
    }
    $detail = 'GUI process survived startup and exposed a top-level window on this Windows runner. Manual visual/functional acceptance remains NOT_VERIFIED.'
} catch {
    if ($failureKind -eq 'NONE') { $failureKind = 'UNEXPECTED_ERROR' }
    $status = if ($failureKind -eq 'WINDOW_NOT_OBSERVED') { 'INCONCLUSIVE' } else { 'FAIL' }
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
        processId = $processId
        processSessionId = $sessionId
        userInteractive = [Environment]::UserInteractive
        failureKind = $failureKind
        status = $status
        detail = $detail
        mainWindowHandle = $windowHandle
        mainWindowTitle = $windowTitle
        windowBounds = $windowRect
        screenshotStatus = $screenshotStatus
        exitCode = $exitCode
        manualGuiAcceptance = 'NOT_VERIFIED'
        independentAutocadInterop = 'NOT_VERIFIED'
    }
    $record | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $report -Encoding utf8
    Write-Host ($record | ConvertTo-Json -Depth 5)
    if (-not [string]::IsNullOrEmpty($env:GITHUB_STEP_SUMMARY)) {
        @("### CAD 360 V1.025 GUI startup smoke", "- Result: $status", "- Failure kind: $failureKind", "- Binary SHA256: $($record.exeSha256)", '- GUI manual acceptance: NOT_VERIFIED', '- Independent AutoCAD roundtrip: NOT_VERIFIED') | Out-File $env:GITHUB_STEP_SUMMARY -Append -Encoding utf8
    }
}
