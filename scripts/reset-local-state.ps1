[CmdletBinding(SupportsShouldProcess = $true)]
param()

$ErrorActionPreference = "Stop"

$AppEntries = @(
    @{
        AppId = "com.vibecal.desktop"
        AppName = "VibeCal"
        ProcessPattern = "*vibecal.exe"
    },
    @{
        AppId = "com.local.applecalendardesktop"
        AppName = "Apple Calendar Desktop"
        ProcessPattern = "*apple-calendar-desktop.exe"
    }
)

$RunKeyPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"

Write-Host "Resetting local state for VibeCal..."

$runningProcesses = @()
foreach ($entry in $AppEntries) {
    $runningProcesses += Get-Process -ErrorAction SilentlyContinue | Where-Object {
        try {
            $_.Path -like $entry.ProcessPattern
        } catch {
            $false
        }
    }
}

if ($runningProcesses | Select-Object -First 1) {
    throw "Close VibeCal and any legacy Apple Calendar Desktop builds before running this script."
}

foreach ($entry in $AppEntries) {
    $localDataPath = Join-Path $env:LOCALAPPDATA $entry.AppId

    if (Test-Path -LiteralPath $localDataPath) {
        if ($PSCmdlet.ShouldProcess($localDataPath, "Remove local app data")) {
            Remove-Item -LiteralPath $localDataPath -Recurse -Force
            Write-Host "Removed local data: $localDataPath"
        }
    } else {
        Write-Host "Local data path not found: $localDataPath"
    }

    if (Get-ItemProperty -Path $RunKeyPath -Name $entry.AppName -ErrorAction SilentlyContinue) {
        if ($PSCmdlet.ShouldProcess($RunKeyPath, "Remove auto start entry")) {
            Remove-ItemProperty -Path $RunKeyPath -Name $entry.AppName
            Write-Host "Removed auto start entry: $($entry.AppName)"
        }
    } else {
        Write-Host "Auto start entry not found: $($entry.AppName)"
    }
}

Write-Host "Reset complete."
