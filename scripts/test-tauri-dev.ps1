# Ordinary PM3 development ownership fixtures; never touch a real service.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/tauri-dev-supervision.ps1"
. "$PSScriptRoot/tauri-dev-session.ps1"
$script:checks = 0
function Assert-That($Condition, [string]$Message) {
    if (!$Condition) { throw "Assertion failed: $Message" }; $script:checks++
}
function Expect-Failure([scriptblock]$Action, [string]$Pattern) {
    $caught = $null
    try { & $Action } catch { $caught = $_.Exception.Message }
    Assert-That ($caught -and $caught -like "*$Pattern*") "Expected $Pattern; got $caught"
}
$config = @{Service='moveup/desk'}
function New-Fixture {
    $script:fixture = @{
        State='running'; ProcessId=42; Events=[Collections.Generic.List[string]]::new()
        Instances=@([pscustomobject]@{Id=42;Path='C:\fixture\release\desk.exe';StartTime=[datetime]'2026-10-09'})
        FailDev=$false; FailRegister=$false; RefuseStop=$false; BadAck=$false; LeaveDebug=$false
        ForeignPid=$false; ForeignCmd=$false; FailStart=$false; StartCount=0
    }
    $script:ops = @{
        Enumerate = { $fixture.Instances }
        Confirm = { $true }
        Validate = { param($existing) $fixture.Events.Add("validate:$($existing.Id)") }
        Register = { $fixture.Events.Add('register'); if ($fixture.FailRegister) { throw 'register failed' } }
        Dev = { $fixture.Events.Add('dev'); if ($fixture.LeaveDebug) { $fixture.Instances=@([pscustomobject]@{Id=99}) }; if ($fixture.FailDev) { throw 'dev failed' } }
        Transport = {
            param($cfg, $argv, $timeout)
            $action=$argv[0]; $fixture.Events.Add($action)
            if ($action -eq 'list') {
                $entry=@{name=$cfg.Service;status=$fixture.State;pid=$fixture.ProcessId}
                if ($fixture.ForeignPid) { $entry.pid=123 }
                return @{Exit=0;Body=(@{services=@($entry)} | ConvertTo-Json -Depth 5 -Compress)}
            }
            if ($action -eq 'describe') {
                $exe=if ($fixture.ForeignCmd) {'C:\foreign\desk.exe'} else {'C:\fixture\release\desk.exe'}
                return @{Exit=0;Body=(@{name=$cfg.Service;cmd=@($exe,'--minimized')} | ConvertTo-Json -Compress)}
            }
            if ($action -eq 'stop') {
                if ($fixture.RefuseStop) { return @{Exit=1;Body='refused'} }
                $fixture.State='stopped'; $fixture.ProcessId=$null; $fixture.Instances=@()
                return @{Exit=0;Body=$(if ($fixture.BadAck) {'unknown'} else {'desk: stopped'})}
            }
            if ($action -eq 'start') {
                if ($fixture.FailStart) { return @{Exit=1;Body='restore refused'} }
                $fixture.StartCount++; $fixture.State='running'; $fixture.ProcessId=84
                $fixture.Instances=@([pscustomobject]@{Id=84;Path='C:\fixture\release\desk.exe'})
                return @{Exit=0;Body='desk: running (pid 84)'}
            }
            throw "Unsupported fixture action $action"
        }
    }
}
function Run-Session($Supervision=$config) {
    Invoke-MoveUpDevSession -Operations $ops -Config $Supervision -ReleaseExe 'C:\fixture\release\desk.exe' -DebugExe 'C:\fixture\debug\desk.exe' -InstallRoot 'C:\fixture\installed' -Force
}
try {
    New-Fixture; Run-Session
    Assert-That ($fixture.StartCount -eq 1) 'restore the replaced PM3 instance once'
    Assert-That ($fixture.Events.IndexOf('stop') -lt $fixture.Events.IndexOf('dev')) 'stop before development'
    Assert-That ($fixture.Events.IndexOf('validate:42') -lt $fixture.Events.IndexOf('stop')) 'validate live identity before stopping'
    New-Fixture; $fixture.FailDev=$true
    Expect-Failure {Run-Session} 'dev failed'
    Assert-That ($fixture.StartCount -eq 1) 'development failure restores only the replaced instance'
    New-Fixture; $fixture.FailRegister=$true
    Expect-Failure {Run-Session} 'register failed'
    Assert-That ($fixture.StartCount -eq 1) 'registration failure restores replaced instance'
    foreach ($failure in @('FailDev', 'FailRegister')) {
        New-Fixture; $fixture[$failure]=$true
        if ($failure -eq 'FailDev') { $fixture.LeaveDebug=$true } else { $fixture.FailStart=$true }
        $combined=$null
        try { Run-Session } catch { $combined=$_.Exception }
        Assert-That ($combined -is [AggregateException]) 'body and restoration failures remain separate exceptions'
        $bodyMessage=if ($failure -eq 'FailDev') { 'dev failed' } else { 'register failed' }
        $restoreMessage=if ($failure -eq 'FailDev') { 'avoid a duplicate' } else { 'PM3 start failed' }
        Assert-That ($combined.InnerExceptions[0].Message -eq $bodyMessage) 'original body failure is retained'
        Assert-That ($combined.InnerExceptions[1].Message -like "*$restoreMessage*") 'restoration failure is retained'
        Assert-That ($combined.Message -like "*$bodyMessage*" -and $combined.Message -like "*$restoreMessage*") 'both failures are visible to the caller'
        Assert-That ($fixture.StartCount -eq 0) 'combined failure never starts another process'
    }
    New-Fixture; $fixture.State='stopped'; $fixture.ProcessId=$null; $fixture.Instances=@()
    Run-Session
    Assert-That ($fixture.StartCount -eq 0 -and $fixture.Events -notcontains 'stop') 'already stopped user choice remains stopped'
    New-Fixture; $fixture.State='restarting'; $fixture.ProcessId=$null; $fixture.Instances=@()
    Expect-Failure {Run-Session} 'not stopped'
    Assert-That ($fixture.StartCount -eq 0 -and $fixture.Events -notcontains 'dev') 'backoff is not a verified replaceable instance'
    foreach ($flag in @('ForeignPid','ForeignCmd','RefuseStop','BadAck')) {
        New-Fixture; $fixture[$flag]=$true
        Expect-Failure {Run-Session} $(switch ($flag) {'ForeignPid' {'not the selected'} 'ForeignCmd' {'identity mismatch'} 'RefuseStop' {'stop failed'} default {'acknowledgement unknown'}})
        Assert-That ($fixture.StartCount -eq 0 -and $fixture.Events -notcontains 'dev') "$flag never starts another process"
    }
    New-Fixture; $fixture.LeaveDebug=$true
    Expect-Failure {Run-Session} 'avoid a duplicate'
    Assert-That ($fixture.StartCount -eq 0) 'leftover debug instance prevents restoration'
    New-Fixture; $fixture.Instances += $fixture.Instances[0]
    Expect-Failure {Run-Session} 'Multiple desk'
    Assert-That ($fixture.Events.Count -eq 0) 'ambiguous instances untouched'
    New-Fixture; Run-Session $null
    throw 'Expected the unowned instance to be refused'
} catch {
    # The unowned-instance case is verified separately below.
    if ($_.Exception.Message -notlike '*no verified PM3 owner*') { Write-Error $_ -ErrorAction Continue; exit 1 }
}
try {
    Assert-That ($fixture.StartCount -eq 0 -and $fixture.Events.Count -eq 0) 'unowned instance untouched'
    New-Fixture; $fixture.Instances=@(); Run-Session $null
    Assert-That ($fixture.Events -contains 'dev' -and $fixture.StartCount -eq 0) 'unmanaged fresh development creates no restoration'
    $testRoot=Join-Path (Split-Path -Parent $PSScriptRoot) 'src-tauri/target'
    $temp=Join-Path $testRoot ('ordinary-pm3-fixture-' + [Guid]::NewGuid())
    $releaseDir=Join-Path $temp 'releases'
    [IO.Directory]::CreateDirectory($releaseDir) | Out-Null
    $launcher=Join-Path $releaseDir 'launch.mjs'
    try {
        [IO.File]::WriteAllText($launcher, "const [a,s,...args]=process.argv.slice(2); if(s==='fixture/timeout')setInterval(()=>{},1000); else if(s==='fixture/overflow')process.stdout.write('x'.repeat(131072)); else console.log(JSON.stringify({a,s,args,home:process.env.PM3_HOME}));")
        $node=(Get-Command node -CommandType Application | Select-Object -First 1).Source
        $transport=@{Node=$node;Home=$temp;Service='fixture/desk'}
        $reply=Invoke-MoveUpPm3 $transport @('stop','fixture/desk')
        $value=$reply.Body | ConvertFrom-Json -AsHashtable
        Assert-That ($reply.Exit -eq 0 -and $value.a -ceq 'stop' -and $value.args.Count -eq 0 -and $value.home -ceq $temp) 'real hidden transport preserves ordinary argv and PM3 home'
        Expect-Failure {Invoke-MoveUpPm3 $transport @('stop','fixture/overflow')} 'output limit'
        Expect-Failure {Invoke-MoveUpPm3 $transport @('stop','fixture/timeout') 150} 'timed out'
    } finally {
        Remove-Item -LiteralPath $launcher -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $releaseDir -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $temp -ErrorAction SilentlyContinue
    }
    Write-Host "PASS: $script:checks assertions; synthetic operations only"
} catch { Write-Error $_ -ErrorAction Continue; exit 1 }
