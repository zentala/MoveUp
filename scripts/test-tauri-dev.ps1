# Self-contained fixture runner. Never enumerates/stops real desk processes.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/tauri-dev-supervision.ps1"
. "$PSScriptRoot/tauri-dev-session.ps1"
$script:checks = 0
function Assert-That($Condition, [string]$Message) {
    if (!$Condition) { throw "Assertion failed: $Message" }
    $script:checks++
}
function Expect-Failure([scriptblock]$Action, [string]$Pattern) {
    $caught = $null
    try { & $Action } catch { $caught = $_.Exception.Message }
    Assert-That ($caught -and $caught -like "*$Pattern*") "Expected failure: $Pattern; got $caught"
}
function New-Fixture {
    $script:fixture = @{
        Holds = @{ 'user-quit' = 'pre-existing' }; Calls = [Collections.Generic.List[object]]::new()
        Events = [Collections.Generic.List[string]]::new(); Lost = $false; Refuse = $false
        Unknown = $false; FailDev = $false; FailRegister = $false; ResumeLost = $false
        Instances = @([pscustomobject]@{ Id = 42; Path = 'C:\fixture\release\desk.exe'; StartTime = [datetime]'2026-10-09' })
    }
    $script:ops = @{
        Enumerate = { $fixture.Instances }
        Confirm = { $true }
        Stop = { param($instance) $fixture.Events.Add("stop:$($instance.Id)") }
        Register = { $fixture.Events.Add('register'); if ($fixture.FailRegister) { throw 'register failed' } }
        Dev = { $fixture.Events.Add("dev:$env:DESK_REMOTE_PORT"); if ($fixture.FailDev) { throw 'dev failed' } }
        Restore = { param($path) $fixture.Events.Add("restore:$path") }
        Transport = {
            param($config, $argv, $timeout)
            $fixture.Calls.Add(@($argv))
            $action = $argv[0]
            $intentIndex = [array]::IndexOf($argv, '--intent')
            $intent = if ($intentIndex -ge 0) { $argv[$intentIndex + 1] } else { '' }
            $v = $null
            if ($action -eq 'suspend') {
                $fixture.Events.Add('suspend')
                if ($fixture.Refuse) { return @{ Exit = 1; Body = '{"outcome":"refused"}' } }
                $fixture.Holds[$intent] = 'development'
                if ($fixture.Lost -or $fixture.Unknown) { return @{ Exit = 3; Body = '{"outcome":"unknown"}' } }
                $v = @{name=$config.Service; suspended=$true; scope='login'; loginId='L'; intentId=$intent; acknowledgedAt='now'; holds=$fixture.Holds.Count}
            } elseif ($action -eq 'supervision') {
                $mode = if ($fixture.Unknown) { 'unknown' } else { 'suspended' }
                $holds = @($fixture.Holds.Keys | ForEach-Object { @{intentId=$_; acknowledgedAt='now'} })
                $v = @{state='known'; name=$config.Service; supervision=@{mode=$mode; ownedChild=$false; observedLive=$false; activeCandidate=$null; promotedCandidate=$null; misses=0; demoted=@(); suspension=@{loginId='L'; holds=$holds}}}
            } elseif ($action -eq 'resume') {
                $fixture.Events.Add('resume')
                $released = $fixture.Holds.ContainsKey($intent)
                $fixture.Holds.Remove($intent)
                if ($fixture.ResumeLost) { $fixture.ResumeLost=$false; return @{Exit=3; Body='{}'} }
                $v = @{name=$config.Service; suspended=($fixture.Holds.Count -gt 0); released=$released; holds=$fixture.Holds.Count; status='suspended'}
            } else { throw "Unexpected action $action" }
            return @{ Exit=0; Body=($v | ConvertTo-Json -Depth 8 -Compress) }
        }
    }
}
$config = @{Service='moveup/desk'}
function Run-Session($Supervision = $config) {
    Invoke-MoveUpDevSession -Operations $ops -Config $Supervision -ReleaseExe 'C:\fixture\release\desk.exe' -DebugExe 'C:\fixture\debug\desk.exe' -InstallRoot 'C:\fixture\installed' -Force
}
try {
    New-Fixture
    Run-Session
    Assert-That ($fixture.Events[0] -eq 'suspend' -and $fixture.Events[1] -eq 'stop:42') 'suspend precedes replacement'
    Assert-That ($fixture.Holds.Count -eq 1 -and $fixture.Holds.ContainsKey('user-quit')) 'preserve user hold'
    Assert-That ($fixture.Events[-1] -eq 'resume') 'finally cleanup'
    $suspendIntent = $fixture.Calls[0][3]
    Assert-That ($fixture.Calls[-1][3] -eq $suspendIntent -and $fixture.Calls[-1] -notcontains '--all') 'resume own intent only'
    Assert-That ($fixture.Events -notmatch '^restore:') 'PM3 owns supervised restoration'

    New-Fixture; $fixture.FailDev=$true
    Expect-Failure { Run-Session } 'dev failed'
    Assert-That ($fixture.Holds.Count -eq 1 -and $fixture.Events[-1] -eq 'resume') 'failed dev releases own hold'
    New-Fixture; $fixture.FailRegister=$true
    Expect-Failure { Run-Session } 'register failed'
    Assert-That ($fixture.Events[-1] -eq 'resume') 'registration failure is inside finally'

    New-Fixture; $fixture.Refuse=$true
    Expect-Failure { Run-Session } 'suspension UNKNOWN'
    Assert-That ($fixture.Events -notcontains 'stop:42' -and $fixture.Events -notcontains 'register') 'refused suspension prevents replacement'
    Assert-That ($fixture.Calls[-1] -notcontains '--start') 'failed suspend cleanup does not start'
    New-Fixture; $fixture.Lost=$true
    Run-Session
    Assert-That ($fixture.Calls[1][0] -eq 'supervision' -and $fixture.Events -contains 'stop:42') 'lost ack readback authorizes replacement'
    New-Fixture; $fixture.Unknown=$true
    Expect-Failure { Run-Session } 'suspension UNKNOWN'
    Assert-That ($fixture.Holds.Count -eq 1 -and $fixture.Events -notcontains 'stop:42') 'unknown applicability prevents replacement and releases own hold'
    New-Fixture; $fixture.ResumeLost=$true
    Run-Session
    Assert-That ($fixture.Calls[-2][3] -eq $fixture.Calls[-1][3]) 'lost resume retries same intent'

    $oldPort=$env:DESK_REMOTE_PORT
    try {
        $env:DESK_REMOTE_PORT='4499'
        New-Fixture; Run-Session
        Assert-That ($fixture.Events[0] -eq 'suspend' -and $fixture.Events -contains 'dev:4499') 'alternate port still suspends'
    } finally { $env:DESK_REMOTE_PORT=$oldPort }

    New-Fixture; $fixture.Holds['other-dev']='nested'
    Run-Session
    Assert-That ($fixture.Holds.Count -eq 2 -and $fixture.Holds.ContainsKey('other-dev')) 'nested launcher hold survives'
    New-Fixture; Run-Session $null
    Assert-That ($fixture.Calls.Count -eq 0 -and $fixture.Events[-1] -like 'restore:*') 'ordinary bridge restores release'
    New-Fixture; $fixture.FailDev=$true
    Expect-Failure { Run-Session $null } 'dev failed'
    Assert-That ($fixture.Events[-1] -like 'restore:*') 'bridge restores release on failure'
    New-Fixture; $fixture.Instances += $fixture.Instances[0]
    Expect-Failure { Run-Session } 'Multiple desk'
    Assert-That ($fixture.Calls.Count -eq 0 -and $fixture.Events.Count -eq 0) 'ambiguous instances untouched'
    New-Fixture; $fixture.Instances[0].Path='C:\unrelated\desk.exe'
    Expect-Failure { Run-Session } 'Unrecognized desk'
    Assert-That ($fixture.Events.Count -eq 0) 'unknown path untouched'

    foreach ($body in @('{}', '{', '{"state":"known","name":"other"}', '{"state":"known","name":"moveup/desk","supervision":{}}')) {
        Assert-That (!(Test-MoveUpHold @{Exit=0; Body=$body} 'supervision' 'moveup/desk' 'quit')) 'malformed status is unknown'
    }
    $saved = @{}
    $keys = @('DESK_SUPERVISION','DESK_PM3_SERVICE','PM3_SERVICE','DESK_PM3_NODE','DESK_PM3_HOME')
    try {
        foreach ($key in $keys) { $saved[$key]=[Environment]::GetEnvironmentVariable($key); [Environment]::SetEnvironmentVariable($key,$null) }
        Assert-That ($null -eq (Get-MoveUpSupervision)) 'unconfigured is ordinary bridge'
        $env:DESK_SUPERVISION='pm3'
        Expect-Failure { Get-MoveUpSupervision } 'project/service'
        $env:DESK_SUPERVISION='typo'
        Expect-Failure { Get-MoveUpSupervision } 'pm3 or bridge'
    } finally { foreach ($key in $keys) { [Environment]::SetEnvironmentVariable($key,$saved[$key]) } }
    # Exercise the real hidden Node boundary against a fixture, never PM3.
    $testRoot = Join-Path (Split-Path -Parent $PSScriptRoot) 'src-tauri/target'
    $temp = Join-Path $testRoot ('supervision-fixture-' + [Guid]::NewGuid())
    $releaseDir = Join-Path $temp 'releases'
    $launcher = Join-Path $releaseDir 'launch.mjs'
    $fixtureKeys = @('PM3_TEST_IPC_PATH','PM3_TEST_RUN_DIR','PM3_TEST_PIDFILE_PATH')
    $savedFixture = @{}
    [IO.Directory]::CreateDirectory($releaseDir) | Out-Null
    try {
        foreach ($key in $fixtureKeys) {
            $savedFixture[$key]=[Environment]::GetEnvironmentVariable($key)
            [Environment]::SetEnvironmentVariable($key, "synthetic-inherited-$key")
        }
        Copy-Item -LiteralPath "$PSScriptRoot/../tests/fixtures/supervision-launcher.mjs" -Destination $launcher
        $node = if ($env:PM3_DAEMON_NODE) { $env:PM3_DAEMON_NODE } else { $env:DESK_PM3_NODE }
        Assert-That ($node -and [IO.Path]::IsPathFullyQualified($node)) 'explicit fixture Node supplied'
        $transportConfig = @{Node=$node; Home=$temp; Service='fixture/desk'}
        $reply = Invoke-MoveUpPm3 $transportConfig @('suspend','fixture/desk','--intent','quit','--json')
        Assert-That (Test-MoveUpHold $reply 'suspend' 'fixture/desk' 'quit') 'actual hidden transport acknowledgement'
        $reply = Invoke-MoveUpPm3 $transportConfig @('suspend','fixture/argv','--intent','space & $ literal','--json')
        $v = $reply.Body | ConvertFrom-Json -AsHashtable
        Assert-That ($v.args[1] -ceq 'space & $ literal' -and $v.home -ceq $temp) 'structured argv and explicit PM3_HOME'
        foreach ($pair in @(@('ipc','PM3_TEST_IPC_PATH'), @('run','PM3_TEST_RUN_DIR'), @('pidfile','PM3_TEST_PIDFILE_PATH'))) {
            Assert-That ($v[$pair[0]] -ceq [Environment]::GetEnvironmentVariable($pair[1])) 'fixture environment inherits'
        }
        Expect-Failure { Invoke-MoveUpPm3 $transportConfig @('suspend','fixture/output') } 'output limit'
        Expect-Failure { Invoke-MoveUpPm3 $transportConfig @('suspend','fixture/stderr') } 'output limit'
        Expect-Failure { Invoke-MoveUpPm3 $transportConfig @('suspend','fixture/timeout') 150 } 'timed out'
    } finally {
        foreach ($key in $fixtureKeys) { [Environment]::SetEnvironmentVariable($key,$savedFixture[$key]) }
        Remove-Item -LiteralPath $launcher -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $releaseDir -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $temp -ErrorAction SilentlyContinue
    }
    Write-Host "PASS: $script:checks assertions; synthetic operations only"
    exit 0
} catch {
    Write-Error "FAIL after $script:checks assertions: $_" -ErrorAction Continue
    exit 1
}
