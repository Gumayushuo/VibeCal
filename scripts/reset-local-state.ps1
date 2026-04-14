[CmdletBinding(SupportsShouldProcess = $true)]
param()

$ErrorActionPreference = "Stop"

$AppId = "com.local.applecalendardesktop"
$AppName = "Apple Calendar Desktop"
$LocalDataPath = Join-Path $env:LOCALAPPDATA $AppId
$RunKeyPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"

Write-Host "Resetting local state for $AppName..."

$runningProcesses = Get-Process -ErrorAction SilentlyContinue | Where-Object {
    try {
        $_.Path -like "*apple-calendar-desktop.exe"
    } catch {
        $false
    }
}

if ($runningProcesses) {
    throw "Close Apple Calendar Desktop before running this script."
}

if (Test-Path -LiteralPath $LocalDataPath) {
    if ($PSCmdlet.ShouldProcess($LocalDataPath, "Remove local app data")) {
        Remove-Item -LiteralPath $LocalDataPath -Recurse -Force
        Write-Host "Removed local data: $LocalDataPath"
    }
} else {
    Write-Host "Local data path not found: $LocalDataPath"
}

if (Get-ItemProperty -Path $RunKeyPath -Name $AppName -ErrorAction SilentlyContinue) {
    if ($PSCmdlet.ShouldProcess($RunKeyPath, "Remove auto start entry")) {
        Remove-ItemProperty -Path $RunKeyPath -Name $AppName
        Write-Host "Removed auto start entry: $AppName"
    }
} else {
    Write-Host "Auto start entry not found: $AppName"
}

Write-Host "Reset complete."
