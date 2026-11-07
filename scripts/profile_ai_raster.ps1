# Windows AI large-image test profiler. Run from CADTBT360_V1.000 workspace.
# Only observed test-worker RSS is reported; no synthetic "PASS" on missing samples.
param(
    [Parameter(Mandatory=$true)][string]$TestName,
    [int]$PollMilliseconds = 200
)
$ErrorActionPreference = 'Stop'
if ($PollMilliseconds -lt 50 -or $PollMilliseconds -gt 2000) { throw 'Invalid poll interval' }
$timer = [System.Diagnostics.Stopwatch]::StartNew()
$peakWorkingSet = [long]0
$samples = 0
$proc = Start-Process -FilePath 'cargo' -ArgumentList @('test','--locked','-p','cadcraft-mcp','--lib',$TestName,'--','--nocapture') -PassThru -NoNewWindow
try {
    do {
        # cargo uses a hashed exe name such as cadcraft_mcp-<hash>.exe.
        # Count only the test worker, not rustc or the cargo compilation phase.
        foreach ($worker in @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -match '^cadcraft_mcp' })) {
            try {
                $worker.Refresh()
                $rss = [long]$worker.WorkingSet64
                if ($rss -gt $peakWorkingSet) { $peakWorkingSet = $rss }
                $samples++
            } catch {
                # The test process may exit between enumeration and sampling.
            }
        }
        Start-Sleep -Milliseconds $PollMilliseconds
        $proc.Refresh()
    } while (-not $proc.HasExited)
    $proc.WaitForExit()
    $timer.Stop()
    $elapsedMs=[math]::Round($timer.Elapsed.TotalMilliseconds,0)
    if ($samples -gt 0) {
        $peakMiB=[math]::Round($peakWorkingSet / 1MB,2)
        Write-Host "AI_BENCHMARK test=$TestName elapsed_ms=$elapsedMs sampled_worker_peak_rss_mib=$peakMiB samples=$samples"
    } else {
        Write-Warning "AI_BENCHMARK test=$TestName elapsed_ms=$elapsedMs peak_rss=NOT_CAPTURED (worker completed between samples)"
    }
    if ($proc.ExitCode -ne 0) { throw "AI test $TestName failed with exit code $($proc.ExitCode)" }
} finally {
    if (-not $proc.HasExited) {
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }
    $proc.Dispose()
}
