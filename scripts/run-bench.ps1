param(
    [string]$Root = (Get-Location).Path,
    [string]$ResultsDir = "bench-results",
    [string]$Concurrency = "1,5,10,25,50",
    [int]$Requests = 100,
    [int]$Warmup = 10,
    [string]$InputImage = ".\input.png",
    [string[]]$Services = @("spin", "docker"),
    [string[]]$Scenarios = @(
        "health",
        "validate-json",
        "validate-json-schema",
        "json-to-csv",
        "csv-to-json",
        "image-metadata",
        "image-grayscale",
        "image-resize"
    ),

    # Używamy 127.0.0.1 zamiast localhost, bo na Windowsie localhost
    # czasem rozwiązuje się przez IPv6 ::1, a usługa słucha tylko na IPv4.
    [string]$SpinUrl = "http://127.0.0.1:3000",
    [string]$DockerUrl = "http://127.0.0.1:8081"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$BenchPy = Join-Path $ScriptDir "bench_http.py"

if (-not (Test-Path $BenchPy)) {
    throw "bench_http.py not found at: $BenchPy"
}

$OutDir = Join-Path $Root $ResultsDir
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

function Test-HttpOk {
    param(
        [string]$Url,
        [int]$TimeoutSeconds = 3
    )

    try {
        $output = & curl.exe `
            --silent `
            --show-error `
            --output NUL `
            --write-out "%{http_code}" `
            --max-time $TimeoutSeconds `
            $Url 2>$null

        $statusCode = 0
        [void][int]::TryParse($output, [ref]$statusCode)

        return ($statusCode -ge 200 -and $statusCode -lt 300)
    }
    catch {
        return $false
    }
}

function Assert-ServiceAvailable {
    param(
        [string]$Name,
        [string]$BaseUrl
    )

    $healthUrl = "$BaseUrl/health"

    Write-Host "Checking $Name at $healthUrl ..."

    if (-not (Test-HttpOk $healthUrl)) {
        Write-Host ""
        Write-Host "$Name health check failed." -ForegroundColor Red
        Write-Host "Try manually:" -ForegroundColor Yellow
        Write-Host "  curl.exe $healthUrl"
        Write-Host ""
        throw "$Name does not respond at $healthUrl"
    }

    Write-Host "$Name OK" -ForegroundColor Green
}

Write-Host "Root:        $Root"
Write-Host "Results:     $OutDir"
Write-Host "Spin URL:    $SpinUrl"
Write-Host "Docker URL:  $DockerUrl"
Write-Host "Concurrency: $Concurrency"
Write-Host "Requests:    $Requests"
Write-Host "Warmup:      $Warmup"
Write-Host ""

foreach ($service in $Services) {
    if ($service -eq "spin") {
        Assert-ServiceAvailable -Name "Spin" -BaseUrl $SpinUrl
        $baseUrl = $SpinUrl
    }
    elseif ($service -eq "docker") {
        Assert-ServiceAvailable -Name "Docker" -BaseUrl $DockerUrl
        $baseUrl = $DockerUrl
    }
    else {
        throw "Unknown service: $service"
    }

    foreach ($scenario in $Scenarios) {
        $argsList = @(
            $BenchPy,
            "--service", $service,
            "--base-url", $baseUrl,
            "--scenario", $scenario,
            "--concurrency", $Concurrency,
            "--requests", "$Requests",
            "--warmup", "$Warmup",
            "--out-dir", $OutDir
        )

        if ($scenario -like "image-*") {
            $imagePath = Resolve-Path $InputImage -ErrorAction Stop
            $argsList += @("--input-image", $imagePath.Path)
        }

        Write-Host ""
        Write-Host "Running $service / $scenario ..."
        python @argsList
    }
}

Write-Host ""
Write-Host "Done. CSV files saved in: $OutDir"