<#
.SYNOPSIS
    Guarded Windows development session; PM3 owns recovery when configured.
#>
param([switch]$Live, [switch]$Mock, [switch]$Demo, [switch]$Force)
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) {
    $pwsh = Get-Command pwsh -ErrorAction Stop
    $forward = @()
    foreach ($key in $PSBoundParameters.Keys) { if ($PSBoundParameters[$key]) { $forward += "-$key" } }
    & $pwsh.Source -NoProfile -File $PSCommandPath @forward
    exit $LASTEXITCODE
}
. "$PSScriptRoot/tauri-dev-supervision.ps1"
. "$PSScriptRoot/tauri-dev-session.ps1"
$repoRoot = Split-Path -Parent $PSScriptRoot
$releaseExe = Join-Path $repoRoot 'src-tauri/target/release/desk.exe'
$debugExe = Join-Path $repoRoot 'src-tauri/target/debug/desk.exe'
$installRoot = Join-Path $env:LOCALAPPDATA 'MoveUp'
$config = Get-MoveUpSupervision
$env:OVERLAY_DATA = if ($Mock) { 'mock' } elseif ($Demo) { 'demo' } else { 'live' }
$operations = @{
    Enumerate = { Get-Process -Name desk -ErrorAction SilentlyContinue }
    Confirm = {
        param($existing)
        Write-Host "Replace PID $($existing.Id), $($existing.Path)? [k/s] (k after 10s)"
        $timer = [Diagnostics.Stopwatch]::StartNew()
        while ($timer.ElapsedMilliseconds -lt 10000) {
            if ([Console]::KeyAvailable) { return [Console]::ReadKey($true).KeyChar -eq 'k' }
            Start-Sleep -Milliseconds 100
        }
        return $true
    }
    Validate = {
        param($existing)
        $current = Get-Process -Id $existing.Id -ErrorAction Stop
        if ($current.Path -ine $existing.Path -or $current.StartTime -ne $existing.StartTime) {
            throw 'Process identity changed; replacement refused.'
        }
    }
    Register = { & "$PSScriptRoot/register-notifications.ps1" }
    Dev = {
        $previous = $env:DESK_SUPERVISION
        try {
            $env:DESK_SUPERVISION = 'bridge' # Parent owns the stopped release and restoration.
            pnpm exec tauri dev
            if ($LASTEXITCODE -ne 0) { throw "Tauri development failed (exit $LASTEXITCODE)" }
        } finally { $env:DESK_SUPERVISION = $previous }
    }
    Transport = { param($configuration, $arguments, $timeout) Invoke-MoveUpPm3 $configuration $arguments $timeout }
}
Invoke-MoveUpDevSession -Operations $operations -Config $config -ReleaseExe $releaseExe -DebugExe $debugExe -InstallRoot $installRoot -Force:$Force
