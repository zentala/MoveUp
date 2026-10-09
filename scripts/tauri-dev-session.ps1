# Injectable session operations. Tests supply synthetic process identities only.
function Invoke-MoveUpDevSession {
    param([hashtable]$Operations, $Config, [string]$ReleaseExe, [string]$DebugExe,
        [string]$InstallRoot, [switch]$Force)
    $intent = 'dev-' + [Guid]::NewGuid().ToString()
    $attempted = $false
    $replaced = $false
    $restore = $null
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
        }
        if ($Config) {
            $attempted = $true # Also clean up a request whose acknowledgement was lost.
            Confirm-MoveUpSuspend $Config $intent $Operations.Transport
        }
        if ($existing) {
            & $Operations.Stop $existing
            $replaced = $true
            if ($path -ieq $ReleaseExe -or $installed) { $restore = $path }
        }
        & $Operations.Register
        & $Operations.Dev
    } finally {
        if ($Config -and $attempted) {
            Release-MoveUpHold $Config $intent $replaced $Operations.Transport
        } elseif ($replaced -and $restore) {
            & $Operations.Restore $restore
        }
    }
}
