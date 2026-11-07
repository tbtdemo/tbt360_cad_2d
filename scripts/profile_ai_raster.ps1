# Windows AI raster benchmark. Invoke from CADTBT360_V1.000.
# Compile is EXCLUDED from timing. Peak WorkingSet64 is only an observed sample.
param(
    [Parameter(Mandatory=$true)][string]$TestName,
    [ValidateRange(5,2000)][int]$PollMilliseconds = 10
)
$ErrorActionPreference = 'Stop'

# Cargo JSON identifies the exact unit-test EXE that belongs to this checkout.
# Never run a newest-file wildcard; it could measure a stale build.
$messages = @(& cargo test --locked -p cadcraft-mcp --lib --no-run --message-format=json)
if ($LASTEXITCODE -ne 0) { throw "AI test compilation failed: cargo code $LASTEXITCODE" }
$executables = @($messages | ForEach-Object {
    $line = [string]$_
    if ($line.StartsWith('{')) {
        try {
            $entry = $line | ConvertFrom-Json
            if ($entry.reason -eq 'compiler-artifact' -and
                $entry.target.name -eq 'cadcraft_mcp' -and
                $entry.profile.test -eq $true -and
                $entry.executable -and
                [string]$entry.executable -like '*.exe') {
                [string]$entry.executable
            }
        } catch {
            # Non-JSON cargo diagnostic; only compiler-artifact entries are used.
        }
    }
} | Select-Object -Unique)
if ($executables.Count -ne 1 -or !(Test-Path -LiteralPath $executables[0] -PathType Leaf)) {
    throw "Expected one compiled cadcraft_mcp test EXE from Cargo JSON; got $($executables.Count)"
}
$exe = [string]$executables[0]
# Rust test filtering succeeds with "0 tests" by default. For a trustworthy
# benchmark require precisely the named test to exist and execute.
$exactTest = "raster_vector::tests::$TestName"
$listed = @(& $exe --list --format=terse)
if ($LASTEXITCODE -ne 0) { throw "Cannot enumerate Rust tests in $exe" }
$matches = @($listed | Where-Object { $_ -eq "$($exactTest): test" })
if ($matches.Count -ne 1) {
    throw "Benchmark requires one real test: $exactTest (found $($matches.Count))"
}
$timer = [System.Diagnostics.Stopwatch]::StartNew()
$peakWorkingSet = [long]0
$samples = 0
$proc = Start-Process -FilePath $exe -ArgumentList @($exactTest,'--exact','--nocapture') -PassThru -NoNewWindow
try {
    do {
        try {
            $proc.Refresh()
            if (-not $proc.HasExited) {
                $rss = [long]$proc.WorkingSet64
                if ($rss -gt $peakWorkingSet) { $peakWorkingSet = $rss }
                $samples++
            }
        } catch {
            # Process can terminate between Refresh and WorkingSet64 sampling.
        }
        if (-not $proc.HasExited) {
            Start-Sleep -Milliseconds $PollMilliseconds
        }
    } while (-not $proc.HasExited)
    $proc.WaitForExit()
    $timer.Stop()
    $wallMs=[math]::Round($timer.Elapsed.TotalMilliseconds,1)
    if ($samples -gt 0) {
        $peakMiB=[math]::Round($peakWorkingSet / 1MB,2)
        Write-Host "AI_BENCHMARK test=$TestName test_exe_wall_ms=$wallMs sampled_test_peak_rss_mib=$peakMiB samples=$samples poll_ms=$PollMilliseconds"
    } else {
        Write-Warning "AI_BENCHMARK test=$TestName test_exe_wall_ms=$wallMs sampled_test_peak_rss_mib=NOT_CAPTURED samples=0"
    }
    if ($proc.ExitCode -ne 0) {
        throw "AI test $TestName failed with exit code $($proc.ExitCode)"
    }
    # Start-Process does not update LASTEXITCODE for its caller.
    $global:LASTEXITCODE = 0
} finally {
    if (-not $proc.HasExited) {
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }
    $proc.Dispose()
}
