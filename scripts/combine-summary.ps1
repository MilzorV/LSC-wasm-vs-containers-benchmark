param(
    [string]$ResultsDir = "bench-results",
    [string]$OutFile = "combined-summary.csv"
)

$ErrorActionPreference = "Stop"

$files = Get-ChildItem -Path $ResultsDir -Filter "summary-*.csv" -File
if (-not $files) {
    throw "No summary-*.csv files found in $ResultsDir"
}

$rows = foreach ($file in $files) {
    Import-Csv $file.FullName
}

$path = Join-Path $ResultsDir $OutFile
$rows | Export-Csv -Path $path -NoTypeInformation -Encoding UTF8

Write-Host "Combined summary saved to: $path"
