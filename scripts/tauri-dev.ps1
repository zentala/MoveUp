<#
.SYNOPSIS
    Tauri dev launcher with process guard (PowerShell version)
.DESCRIPTION
    Replaces tauri-dev.sh for Windows PowerShell environments where
    bash cannot find node/pnpm in PATH.
.EXAMPLE
    .\scripts\tauri-dev.ps1              # live mode (default, real sensor)
    .\scripts\tauri-dev.ps1 -Live        # real sensor data (explicit)
    .\scripts\tauri-dev.ps1 -Mock        # simulated sit/stand
    .\scripts\tauri-dev.ps1 -Force       # auto-kill old instance
    .\scripts\tauri-dev.ps1 -Live -Force # live + auto-kill
#>
param(
    [switch]$Live,
    [switch]$Mock,
    [switch]$Demo,
    [switch]$Force
)

$ProcessName = "desk"
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$ReleaseExe = Join-Path $RepoRoot "src-tauri\target\release\desk.exe"

# --- Overlay data source ---
# Default is ALWAYS live. Demo/Mock require explicit flag.
if ($Mock)      { $env:OVERLAY_DATA = "mock" }
elseif ($Demo)  { $env:OVERLAY_DATA = "demo" }
else            { $env:OVERLAY_DATA = "live" }

# --- Process guard ---
# Killing a running instance without a plan to bring it back cost Pawel's own
# production app ~14h of downtime (2026-09-07/08, an agent ran this script for
# a screenshot and nothing restarted the killed instance afterward — see
# .plan/BACKLOG.md). If the process this guard kills is the RELEASE build --
# i.e. the one Windows autostart actually launches, not a prior dev build --
# relaunch it once this dev session ends, whatever the reason it ended.
$killedReleaseInstance = $false
$existing = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
if ($existing) {
    $existingPid = $existing.Id
    $existingPath = $existing.Path
    Write-Host ""
    Write-Host "========================================"
    Write-Host "  Previous instance detected!"
    Write-Host "  $ProcessName.exe is running (PID: $existingPid)"
    Write-Host "  Path: $existingPath"
    Write-Host "========================================"
    Write-Host ""

    if ($existingPath -and ($existingPath -ieq $ReleaseExe)) {
        $killedReleaseInstance = $true
        Write-Host "  This is the RELEASE build (what autostart launches)."
        Write-Host "  It will be relaunched automatically when this dev session ends."
        Write-Host ""
    }

    if ($Force) {
        Write-Host "  -Force: killing $ProcessName.exe..."
        Stop-Process -Name $ProcessName -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
        Write-Host "  Killed. Starting new build."
        Write-Host ""
    } else {
        Write-Host "  [k] Kill old process and start new one (default in 10s)"
        Write-Host "  [s] Skip - keep old process, abort build"
        Write-Host ""

        # Read with 10s timeout
        $choice = $null
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        Write-Host -NoNewline "  Choice [k/s]: "
        while ($sw.ElapsedMilliseconds -lt 10000) {
            if ([Console]::KeyAvailable) {
                $key = [Console]::ReadKey($true)
                $choice = $key.KeyChar
                Write-Host $choice
                break
            }
            Start-Sleep -Milliseconds 100
        }
        if (-not $choice) {
            $choice = 'k'
            Write-Host "k (timeout)"
        }

        if ($choice -eq 's') {
            Write-Host ""
            Write-Host "  Keeping old instance. Build aborted."
            exit 1
        }

        Write-Host ""
        Write-Host "  Killing $ProcessName.exe (PID: $existingPid)..."
        Stop-Process -Name $ProcessName -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
        Write-Host "  Killed. Starting new build."
        Write-Host ""
    }
}

# --- Ensure notification AUMID is registered (dev mode) ---
& "$PSScriptRoot\register-notifications.ps1"

# --- Launch tauri dev, always try to restore what we killed afterward ---
try {
    pnpm exec tauri dev
} finally {
    if ($killedReleaseInstance) {
        Write-Host ""
        Write-Host "  Dev session ended -- relaunching the release build it replaced."
        Start-Process -FilePath $ReleaseExe -ArgumentList "--minimized"
        Write-Host "  Relaunched: $ReleaseExe --minimized"
    }
}
