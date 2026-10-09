# Public PM3 adapter; dot-sourcing only defines functions.
function Get-MoveUpSupervision {
    if ($env:DESK_SUPERVISION -eq 'bridge') { return $null }
    $service = if ($env:DESK_PM3_SERVICE) { $env:DESK_PM3_SERVICE } else { $env:PM3_SERVICE }
    if (!$service -and !$env:DESK_SUPERVISION -and !$env:DESK_PM3_NODE -and !$env:DESK_PM3_HOME) { return $null }
    if ($env:DESK_SUPERVISION -and $env:DESK_SUPERVISION -ne 'pm3') { throw 'DESK_SUPERVISION must be pm3 or bridge' }
    if ($service -notmatch '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$') { throw 'Set DESK_PM3_SERVICE to project/service' }
    $node = if ($env:DESK_PM3_NODE) { $env:DESK_PM3_NODE } else { $env:PM3_DAEMON_NODE }
    $homePath = if ($env:DESK_PM3_HOME) { $env:DESK_PM3_HOME } elseif ($env:PM3_HOME) { $env:PM3_HOME } else { Join-Path $env:USERPROFILE '.pm3' }
    if (!$node -or ![IO.Path]::IsPathFullyQualified($node) -or ![IO.Path]::IsPathFullyQualified($homePath)) { throw 'PM3 requires absolute Node and home paths' }
    if (!(Test-Path -LiteralPath $node -PathType Leaf) -or !(Test-Path -LiteralPath (Join-Path $homePath 'releases/launch.mjs') -PathType Leaf)) { throw 'PM3 Node or public launcher is missing' }
    return @{ Node = $node; Home = $homePath; Service = $service }
}

function Invoke-MoveUpPm3 {
    param($Config, [string[]]$Arguments, [int]$TimeoutMs = 2000)
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $Config.Node
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.ArgumentList.Add((Join-Path $Config.Home 'releases/launch.mjs'))
    foreach ($arg in $Arguments) { $info.ArgumentList.Add($arg) }
    # Inherit every supplied fixture key, including pipe/run/pidfile overrides.
    $info.Environment['PM3_HOME'] = $Config.Home
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    $started = $false
    try {
        $started = $process.Start()
        if (!$started) { throw 'Could not start PM3 helper' }
        $output = Read-MoveUpPm3Output $process $TimeoutMs
        return @{ Exit = $process.ExitCode; Body = $output }
    } finally {
        if ($started -and !$process.HasExited) { $process.Kill() }
        $process.Dispose()
    }
}

function Read-MoveUpPm3Output {
    param($Process, [int]$TimeoutMs)
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $streams = @($Process.StandardOutput.BaseStream, $Process.StandardError.BaseStream)
    $buffers = @([byte[]]::new(4096), [byte[]]::new(4096))
    $totals = @(0, 0)
    $finished = @($false, $false)
    $tasks = @($streams[0].ReadAsync($buffers[0], 0, 4096), $streams[1].ReadAsync($buffers[1], 0, 4096))
    $stdout = [IO.MemoryStream]::new()
    try {
        while (!($finished[0] -and $finished[1] -and $Process.HasExited)) {
            if ($clock.ElapsedMilliseconds -ge $TimeoutMs) { throw 'PM3 timed out; outcome UNKNOWN' }
            for ($i = 0; $i -lt 2; $i++) {
                if (!$finished[$i] -and $tasks[$i].IsCompleted) {
                    $count = $tasks[$i].GetAwaiter().GetResult()
                    $totals[$i] += $count
                    if ($totals[$i] -gt 65536) { throw 'PM3 output limit exceeded; outcome UNKNOWN' }
                    if ($count -eq 0) { $finished[$i] = $true; continue }
                    if ($i -eq 0) { $stdout.Write($buffers[$i], 0, $count) }
                    $tasks[$i] = $streams[$i].ReadAsync($buffers[$i], 0, 4096)
                }
            }
            Start-Sleep -Milliseconds 10
        }
        return [Text.Encoding]::UTF8.GetString($stdout.ToArray())
    } finally { $stdout.Dispose() }
}

function Test-MoveUpHold {
    param($Response, [string]$Action, [string]$Service, [string]$Intent)
    if ($Response.Exit -ne 0 -or $Response.Body.Length -gt 65536) { return $false }
    try { $v = $Response.Body | ConvertFrom-Json -AsHashtable -ErrorAction Stop } catch { return $false }
    if ($v -isnot [System.Collections.IDictionary] -or $v.name -cne $Service) { return $false }
    if ($Action -eq 'suspend') {
        return ($v.suspended -is [bool] -and $v.suspended -and $v.scope -ceq 'login' -and
            $v.intentId -ceq $Intent -and $v.loginId -is [string] -and $v.loginId.Length -gt 0 -and
            $v.acknowledgedAt -is [string] -and $v.acknowledgedAt.Length -gt 0 -and
            $v.holds -is [long] -and $v.holds -gt 0)
    }
    $sup = $v.supervision
    if ($v.state -cne 'known' -or $sup.mode -cnotin @('owned', 'standing-down', 'awaiting-misses', 'suspended', 'no-candidate', 'degraded')) { return $false }
    if (!(Test-MoveUpStatusShape $sup)) { return $false }
    if ($sup.suspension -isnot [System.Collections.IDictionary] -or $sup.suspension.loginId -isnot [string] -or !$sup.suspension.loginId -or $sup.suspension.holds -isnot [array]) { return $false }
    $matched = $false
    foreach ($hold in $sup.suspension.holds) {
        if ($hold -isnot [System.Collections.IDictionary] -or $hold.intentId -isnot [string] -or !$hold.intentId -or $hold.acknowledgedAt -isnot [string] -or !$hold.acknowledgedAt) { return $false }
        if ($hold.Contains('reason') -and $hold.reason -isnot [string]) { return $false }
        if ($hold.intentId -ceq $Intent) { $matched = $true }
    }
    return $matched
}

function Test-MoveUpStatusShape {
    param($Sup)
    if ($Sup -isnot [System.Collections.IDictionary] -or $Sup.ownedChild -isnot [bool]) { return $false }
    if ($Sup.observedLive -isnot [bool] -and $Sup.observedLive -cne 'unknown') { return $false }
    foreach ($key in @('activeCandidate','promotedCandidate')) {
        if (!$Sup.Contains($key) -or ($null -ne $Sup[$key] -and $Sup[$key] -isnot [string])) { return $false }
    }
    if ($Sup.misses -isnot [long] -or $Sup.misses -lt 0 -or $Sup.demoted -isnot [array]) { return $false }
    if ($Sup.Contains('detail') -and $Sup.detail -isnot [string]) { return $false }
    foreach ($entry in $Sup.demoted) {
        if ($entry -isnot [System.Collections.IDictionary] -or $entry.id -isnot [string] -or $entry.artifact -isnot [string] -or $entry.reason -isnot [string]) { return $false }
    }
    if ($Sup.Contains('persistence') -and ($Sup.persistence.ok -isnot [bool] -or $Sup.persistence.ok -or $Sup.persistence.detail -isnot [string])) { return $false }
    if ($Sup.Contains('lastCallback')) {
        $callback=$Sup.lastCallback
        if ($callback -isnot [System.Collections.IDictionary] -or $callback.candidateId -isnot [string] -or $callback.ok -isnot [bool] -or ($callback.Contains('detail') -and $callback.detail -isnot [string])) { return $false }
    }
    return $true
}

function Confirm-MoveUpSuspend {
    param($Config, [string]$Intent, [scriptblock]$Transport, [int]$BudgetMs = 8000)
    $clock = [Diagnostics.Stopwatch]::StartNew()
    foreach ($action in @('suspend', 'supervision', 'suspend', 'supervision')) {
        $remaining = $BudgetMs - $clock.ElapsedMilliseconds
        if ($remaining -le 0) { break }
        $argv = @($action, $Config.Service)
        if ($action -eq 'suspend') { $argv += @('--intent', $Intent, '--reason', 'MoveUp development') }
        $argv += '--json'
        try {
            $response = & $Transport $Config $argv ([int][Math]::Min(2000, $remaining))
            if (Test-MoveUpHold $response $action $Config.Service $Intent) { return }
        } catch { Write-Verbose $_ }
    }
    throw 'PM3 suspension UNKNOWN or refused. No process was replaced. Check supervision and retry.'
}

function Release-MoveUpHold {
    param($Config, [string]$Intent, [bool]$Start, [scriptblock]$Transport)
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $argv = @('resume', $Config.Service, '--intent', $Intent, '--json')
    if ($Start) { $argv += '--start' }
    for ($i = 0; $i -lt 4; $i++) {
        $remaining = 8000 - $clock.ElapsedMilliseconds
        if ($remaining -le 0) { break }
        try {
            $response = & $Transport $Config $argv ([int][Math]::Min(2000, $remaining))
            $v = $response.Body | ConvertFrom-Json -AsHashtable -ErrorAction Stop
            if ($response.Exit -eq 0 -and $v.name -ceq $Config.Service -and
                $v.released -is [bool] -and $v.suspended -is [bool] -and
                $v.holds -is [long] -and $v.holds -ge 0 -and $v.suspended -eq ($v.holds -gt 0) -and
                $v.status -is [string] -and $v.status.Length -gt 0) {
                if ($v.startError) { throw "Hold released but PM3 restart failed: $($v.startError)" }
                return
            }
        } catch { Write-Verbose $_ }
    }
    throw "Could not confirm release of development hold $Intent. Retry PM3 resume with this exact --intent."
}
