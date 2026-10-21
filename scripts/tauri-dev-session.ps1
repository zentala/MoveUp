# Only an observed live instance owned by the selected PM3 service is replaced.
function Invoke-MoveUpDevSession {
    param([hashtable]$Operations, $Config, [string]$ReleaseExe, [string]$DebugExe,
        [string]$InstallRoot, [switch]$Force)
    $replaced = $false
    $bodyFailure = $null
    try {
        $instances = @(& $Operations.Enumerate)
        if ($instances.Count -gt 1) { throw 'Multiple desk instances found; identify the owner before development.' }
        $existing = if ($instances.Count -eq 1) { $instances[0] } else { $null }
        if ($existing) {
            $path = $existing.Path
            $installed = $path -and $InstallRoot -and ($path.StartsWith(
                $InstallRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar,
                [StringComparison]::OrdinalIgnoreCase)) -and [IO.Path]::GetFileName($path) -ieq 'desk.exe'
            if (!$path -or !$existing.StartTime -or ($path -ine $ReleaseExe -and $path -ine $DebugExe -and !$installed)) {
                throw 'Unrecognized desk process identity; no process was replaced.'
            }
            if (!$Force -and !(& $Operations.Confirm $existing)) { throw 'Keeping existing instance; development aborted.' }
            if (!$Config) { throw 'Existing desk has no verified PM3 owner; close it gracefully before development.' }
        }
        if ($Config) {
            $service = Get-MoveUpPm3Service $Config $Operations.Transport
            if ($existing) {
                Assert-MoveUpPm3Owner $Config $service $existing $Operations.Transport
                & $Operations.Validate $existing
                Confirm-MoveUpPm3Action $Config 'stop' $Operations.Transport
                $replaced = $true
                $stopped = Get-MoveUpPm3Service $Config $Operations.Transport
                if (!$stopped -or $stopped.status -cne 'stopped' -or $null -ne $stopped.pid) { throw 'PM3 stop did not settle; development aborted.' }
            } elseif ($service -and ($service.status -cne 'stopped' -or $null -ne $service.pid)) {
                throw 'Selected PM3 service is not stopped; no observed instance can be safely replaced.'
            }
        }
        if (@(& $Operations.Enumerate).Count -ne 0) { throw 'A desk process is still present; development aborted.' }
        & $Operations.Register
        & $Operations.Dev
    } catch {
        $bodyFailure = $_
    } finally {
        try {
            if ($Config -and $replaced) {
                if (@(& $Operations.Enumerate).Count -ne 0) { throw 'Desk remains after development; PM3 restoration skipped to avoid a duplicate.' }
                $current = Get-MoveUpPm3Service $Config $Operations.Transport
                if (!$current -or $current.status -cne 'stopped' -or $null -ne $current.pid) { throw 'PM3 state changed; restoration skipped to protect the current owner.' }
                Confirm-MoveUpPm3Action $Config 'start' $Operations.Transport
            }
        } catch {
            if ($bodyFailure) {
                throw [AggregateException]::new(
                    "Development failed: $($bodyFailure.Exception.Message) Restoration failed: $($_.Exception.Message)",
                    [Exception[]]@($bodyFailure.Exception, $_.Exception))
            }
            throw
        }
    }
    if ($bodyFailure) { throw $bodyFailure }
}
