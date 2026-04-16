param(
    [Parameter(Mandatory = $true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
$changelogPath = Join-Path $repoRoot "CHANGELOG.md"
$lines = Get-Content $changelogPath
$headerPattern = '^## \[' + [regex]::Escape($Version) + '\]'
$startIndex = -1

for ($i = 0; $i -lt $lines.Count; $i++) {
    if ($lines[$i] -match $headerPattern) {
        $startIndex = $i
        break
    }
}

if ($startIndex -lt 0) {
    throw "Could not find CHANGELOG entry for version $Version."
}

$section = New-Object System.Collections.Generic.List[string]
for ($i = $startIndex + 1; $i -lt $lines.Count; $i++) {
    if ($lines[$i] -match '^## \[') {
        break
    }
    $section.Add($lines[$i].TrimEnd())
}

$body = ($section -join "`n").Trim()
if ([string]::IsNullOrWhiteSpace($body)) {
    $body = "Release $Version"
}

Write-Output $body
