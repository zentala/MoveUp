<#
.SYNOPSIS
    Tauri dev launcher with process guard (PowerShell version)
.DESCRIPTION
    Replaces tauri-dev.sh for Windows PowerShell environments where
    bash cannot find node/pnpm in PATH.
.EXAMPLE
    .\scripts\tauri-dev.ps1              # demo mode (default)
    .\scripts\tauri-dev.ps1 -Live        # real sensor data
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

# --- Overlay data source ---
if ($Live)      { $env:OVERLAY_DATA = "live" }
elseif ($Mock)  { $env:OVERLAY_DATA = "mock" }
elseif ($Demo)  { $env:OVERLAY_DATA = "demo" }
# else: leave unset (defaults to demo in debug, live in release)

# --- Process guard ---
$existing = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
if ($existing) {
    $pid = $existing.Id
    Write-Host ""
    Write-Host "========================================"
    Write-Host "  Previous instance detected!"
    Write-Host "  $ProcessName.exe is running (PID: $pid)"
    Write-Host "========================================"
    Write-Host ""

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
        Write-Host "  Killing $ProcessName.exe (PID: $pid)..."
        Stop-Process -Name $ProcessName -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
        Write-Host "  Killed. Starting new build."
        Write-Host ""
    }
}

# --- Launch tauri dev ---
pnpm exec tauri dev
