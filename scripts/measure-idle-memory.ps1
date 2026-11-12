param(
    [Parameter(Mandatory = $true)][string]$Executable,
    [Parameter(Mandatory = $true)][string]$OutputFile,
    [int]$WarmupSeconds = 45
)
$ErrorActionPreference = 'Stop'
$executablePath = (Resolve-Path -LiteralPath $Executable).Path
$measuredProcess = Start-Process -FilePath $executablePath -WindowStyle Hidden -PassThru

function Read-OwnedProcesses([uint32]$RootId) {
    $rows = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId
    $ownedIds = [System.Collections.Generic.HashSet[uint32]]::new()
    [void]$ownedIds.Add($RootId)
    do {
        $previousCount = $ownedIds.Count
        foreach ($row in $rows) {
            if ($ownedIds.Contains([uint32]$row.ParentProcessId)) { [void]$ownedIds.Add([uint32]$row.ProcessId) }
        }
    } while ($ownedIds.Count -ne $previousCount)
    @(Get-Process -Id @($ownedIds) -ErrorAction SilentlyContinue)
}

try {
    Start-Sleep -Seconds $WarmupSeconds
    if ($measuredProcess.HasExited) { throw "The app exited before sampling: $($measuredProcess.ExitCode)" }
    $samples = @()
    $start = Get-Date
    for ($index = 0; $index -lt 5; $index++) {
        $owned = @(Read-OwnedProcesses $measuredProcess.Id)
        $ownedIds = @($owned | ForEach-Object Id)
        $counters = @(Get-CimInstance Win32_PerfFormattedData_PerfProc_Process | Where-Object { $_.IDProcess -in $ownedIds })
        $samples += [pscustomobject]@{
            ElapsedSeconds = ((Get-Date) - $start).TotalSeconds
            ProcessCount = $owned.Count
            PrivateBytesMiB = ($owned | Measure-Object PrivateMemorySize64 -Sum).Sum / 1MB
            PrivateWorkingSetMiB = ($counters | Measure-Object WorkingSetPrivate -Sum).Sum / 1MB
            CpuSeconds = ($owned | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
            ConverterProcesses = @($owned | Where-Object ProcessName -eq 'filee-cli').Count
        }
        if ($index -lt 4) { Start-Sleep -Seconds 2 }
    }
    $first = $samples[0]; $last = $samples[-1]
    $result = [pscustomobject]@{
        Executable = $executablePath
        WarmupSeconds = $WarmupSeconds
        MeanPrivateBytesMiB = [math]::Round(($samples | Measure-Object PrivateBytesMiB -Average).Average, 2)
        MeanPrivateWorkingSetMiB = [math]::Round(($samples | Measure-Object PrivateWorkingSetMiB -Average).Average, 2)
        IdleCpuPercent = [math]::Round(100 * ($last.CpuSeconds - $first.CpuSeconds) / ($last.ElapsedSeconds - $first.ElapsedSeconds) / [Environment]::ProcessorCount, 3)
        Samples = $samples
    }
    $result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputFile
    $result | Select-Object Executable, MeanPrivateBytesMiB, MeanPrivateWorkingSetMiB, IdleCpuPercent | Format-List
} finally {
    if (!$measuredProcess.HasExited) { Stop-Process -Id $measuredProcess.Id -ErrorAction SilentlyContinue }
}
