<#
.SYNOPSIS
    Registers AppUserModelId for Smart Desk notifications (dev mode).
.DESCRIPTION
    Windows toast notifications require a registered AUMID to display
    the correct app name. Installed builds get this via Start Menu
    shortcut, but dev mode needs manual registration.
    Safe to run multiple times — only writes if not already set.
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/register-notifications.ps1
#>

$AUMID = "io.zntl.desk"
$DisplayName = "Smart Desk"
$RegBase = "HKCU:\Software\Classes\AppUserModelId"
$NotifBase = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings"

# Resolve icon path (relative to this script → ../src-tauri/icons/icon.ico)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$IconPath = (Resolve-Path "$ScriptDir\..\src-tauri\icons\icon.ico" -ErrorAction SilentlyContinue).Path

$aumidPath = "$RegBase\$AUMID"
$notifPath = "$NotifBase\$AUMID"

$changed = $false

# Register display name + icon
if (-not (Test-Path $aumidPath)) {
    New-Item $aumidPath -Force | Out-Null
    New-ItemProperty $aumidPath -Name DisplayName -Value $DisplayName -PropertyType String -Force | Out-Null
    if ($IconPath) {
        New-ItemProperty $aumidPath -Name IconUri -Value $IconPath -PropertyType String -Force | Out-Null
    }
    $changed = $true
} else {
    $current = (Get-ItemProperty $aumidPath -ErrorAction SilentlyContinue).DisplayName
    if ($current -ne $DisplayName) {
        Set-ItemProperty $aumidPath -Name DisplayName -Value $DisplayName
        $changed = $true
    }
    $currentIcon = (Get-ItemProperty $aumidPath -ErrorAction SilentlyContinue).IconUri
    if ($IconPath -and $currentIcon -ne $IconPath) {
        New-ItemProperty $aumidPath -Name IconUri -Value $IconPath -PropertyType String -Force | Out-Null
        $changed = $true
    }
}

# Enable Action Center
if (-not (Test-Path $notifPath)) {
    New-Item $notifPath -Force | Out-Null
    New-ItemProperty $notifPath -Name ShowInActionCenter -Value 1 -PropertyType DWORD -Force | Out-Null
    $changed = $true
} else {
    $current = (Get-ItemProperty $notifPath -ErrorAction SilentlyContinue).ShowInActionCenter
    if ($current -ne 1) {
        Set-ItemProperty $notifPath -Name ShowInActionCenter -Value 1
        $changed = $true
    }
}

if ($changed) {
    Write-Host "  Registered AUMID '$AUMID' as '$DisplayName' (icon: $IconPath)"
} else {
    Write-Host "  AUMID '$AUMID' already registered."
}
