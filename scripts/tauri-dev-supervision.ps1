# Public ordinary PM3 adapter; dot-sourcing only defines functions.
function Get-MoveUpSupervision {
    if ($env:DESK_SUPERVISION -eq 'bridge') { return $null }
    if ($env:DESK_SUPERVISION -and $env:DESK_SUPERVISION -ne 'pm3') { throw 'DESK_SUPERVISION must be pm3 or bridge' }
    $service = if ($env:DESK_PM3_SERVICE) { $env:DESK_PM3_SERVICE } elseif ($env:PM3_SERVICE) { $env:PM3_SERVICE } else { 'moveup/desk' }
    if ($service -notmatch '^[A-Za-z0-9_-]+/[A-Za-z0-9_-]+$') { throw 'Set DESK_PM3_SERVICE to project/service' }
    $node = if ($env:DESK_PM3_NODE) { $env:DESK_PM3_NODE } elseif ($env:PM3_DAEMON_NODE) { $env:PM3_DAEMON_NODE } else {
        (Get-Command node -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1).Source
    }
    $homePath = if ($env:DESK_PM3_HOME) { $env:DESK_PM3_HOME } elseif ($env:PM3_HOME) { $env:PM3_HOME } else { Join-Path $env:USERPROFILE '.pm3' }
    if (!$node -or ![IO.Path]::IsPathFullyQualified($node) -or ![IO.Path]::IsPathFullyQualified($homePath)) { throw 'PM3 requires absolute Node and home paths' }
    if (!(Test-Path -LiteralPath $node -PathType Leaf) -or !(Test-Path -LiteralPath (Join-Path $homePath 'releases/launch.mjs') -PathType Leaf)) {
        if ($env:DESK_SUPERVISION -or $env:DESK_PM3_SERVICE -or $env:DESK_PM3_HOME) { throw 'Configured PM3 Node or public launcher is missing' }
        return $null
    }
    $configFile = $env:DESK_PM3_CONFIG
    if ($configFile -and (![IO.Path]::IsPathFullyQualified($configFile) -or !(Test-Path -LiteralPath $configFile -PathType Leaf))) { throw 'PM3 configuration must be an existing absolute file' }
    return @{ Node = $node; Home = $homePath; Service = $service; ConfigFile = $configFile }
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
    if ($Config.ConfigFile) { $info.Environment['PM3_CONFIG'] = $Config.ConfigFile; $info.WorkingDirectory = Split-Path -Parent $Config.ConfigFile }
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


function Get-MoveUpPm3Service {
    param($Config, [scriptblock]$Transport)
    $response = & $Transport $Config @('list', '--all', '--json') 8000
    if ($response.Exit -ne 0) { throw 'PM3 list failed; no process may be replaced' }
    try { $value = $response.Body | ConvertFrom-Json -AsHashtable -ErrorAction Stop } catch { throw 'PM3 list returned invalid JSON' }
    if (!$value.Contains('services') -or $value.services -isnot [array]) { throw 'PM3 service state unknown' }
    $matches = @($value.services | Where-Object { $_.name -ceq $Config.Service })
    if ($matches.Count -gt 1) { throw 'PM3 service identity ambiguous' }
    if ($matches.Count -eq 0) { return $null }
    $service = $matches[0]
    if ($service.status -notin @('running','starting','stopped','crashed','restarting') -or
        !$service.Contains('pid') -or ($null -ne $service.pid -and ($service.pid -isnot [long] -or $service.pid -le 0))) { throw 'PM3 service state unknown' }
    return $service
}

function Confirm-MoveUpPm3Action {
    param($Config, [string]$Action, [scriptblock]$Transport)
    $response = & $Transport $Config @($Action, $Config.Service) 45000
    if ($response.Exit -ne 0) { throw "PM3 $Action failed; outcome unconfirmed, no automatic retry" }
    $names = @($Config.Service, ($Config.Service -split '/')[1])
    $states = if ($Action -eq 'stop') { 'stopped' } else { '(running|starting)' }
    $confirmed = $false
    foreach ($name in $names) {
        if ($response.Body.Trim() -match ('^' + [regex]::Escape($name) + ': ' + $states + '( \(pid [0-9]+\))?$')) { $confirmed = $true }
    }
    if (!$confirmed) { throw "PM3 $Action acknowledgement unknown; no automatic retry" }
}

function Assert-MoveUpPm3Owner {
    param($Config, $Service, $Existing, [scriptblock]$Transport)
    if (!$Service -or $Service.pid -ne $Existing.Id -or $Service.status -notin @('running','starting')) { throw 'Existing desk is not the selected PM3 instance; left untouched' }
    $response = & $Transport $Config @('describe', $Config.Service, '--json') 8000
    if ($response.Exit -ne 0) { throw 'PM3 describe failed; instance left untouched' }
    try { $value = $response.Body | ConvertFrom-Json -AsHashtable -ErrorAction Stop } catch { throw 'PM3 description unknown' }
    if ($value.name -cne $Config.Service -or $value.cmd -isnot [array] -or $value.cmd.Count -lt 1 -or
        $value.cmd[0] -ine $Existing.Path) { throw 'PM3 executable identity mismatch; instance left untouched' }
}
