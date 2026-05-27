param(
    [ValidateSet("spin", "docker")]
    [string]$Service,

    [string]$Root = (Get-Location).Path,
    [string]$ResultsDir = "bench-results",
    [int]$Repeats = 5,

    [string]$SpinDir = "spin-file-tools-sdk4",
    [string]$DockerImage = "docker-file-tools:latest",
    [string]$DockerContainer = "file-tools-bench",

    [string]$SpinUrl = "http://127.0.0.1:3000",
    [string]$DockerUrl = "http://127.0.0.1:8081",

    [switch]$KillExisting
)

$ErrorActionPreference = "Stop"

$OutDir = Join-Path $Root $ResultsDir
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$OutCsv = Join-Path $OutDir "cold-start-$Service.csv"

if (-not (Test-Path $OutCsv)) {
    "service,repeat,time_to_health_ms,first_health_request_ms,total_cold_health_ms,notes" |
        Set-Content $OutCsv -Encoding UTF8
}

function Stop-PortOwner {
    param([int]$Port)

    if (-not $KillExisting) {
        return
    }

    try {
        $connections = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
        foreach ($c in $connections) {
            try {
                Stop-Process -Id $c.OwningProcess -Force -ErrorAction SilentlyContinue
            } catch {}
        }
    } catch {
        Write-Warning "Could not inspect/kill port $Port. $_"
    }
}

function Test-HttpOk {
    param(
        [string]$Url,
        [int]$TimeoutSeconds = 2
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

function Wait-Health {
    param(
        [string]$Url,
        [int]$TimeoutMs = 60000
    )

    $sw = [System.Diagnostics.Stopwatch]::StartNew()

    while ($sw.ElapsedMilliseconds -lt $TimeoutMs) {
        if (Test-HttpOk $Url 2) {
            return $sw.ElapsedMilliseconds
        }

        Start-Sleep -Milliseconds 50
    }

    throw "Timeout while waiting for $Url"
}

function Measure-OneRequest {
    param([string]$Url)

    $sw = [System.Diagnostics.Stopwatch]::StartNew()

    $output = & curl.exe `
        --silent `
        --show-error `
        --output NUL `
        --write-out "%{http_code}" `
        --max-time 10 `
        $Url

    $sw.Stop()

    $statusCode = 0
    [void][int]::TryParse($output, [ref]$statusCode)

    if ($statusCode -lt 200 -or $statusCode -ge 300) {
        throw "Request failed for $Url with HTTP $statusCode"
    }

    return $sw.ElapsedMilliseconds
}

for ($i = 1; $i -le $Repeats; $i++) {
    Write-Host "Cold start $Service repeat $i/$Repeats"

    if ($Service -eq "spin") {
        Stop-PortOwner -Port 3000

        $spinPath = Join-Path $Root $SpinDir
        if (-not (Test-Path $spinPath)) {
            throw "Spin directory not found: $spinPath"
        }

        $logOut = Join-Path $OutDir "spin-cold-$i.out.log"
        $logErr = Join-Path $OutDir "spin-cold-$i.err.log"

        $proc = Start-Process `
            -FilePath "spin" `
            -ArgumentList @("up", "--listen", "127.0.0.1:3000") `
            -WorkingDirectory $spinPath `
            -RedirectStandardOutput $logOut `
            -RedirectStandardError $logErr `
            -PassThru `
            -WindowStyle Hidden

        $totalSw = [System.Diagnostics.Stopwatch]::StartNew()
        $readyMs = Wait-Health "$SpinUrl/health"
        $firstReqMs = Measure-OneRequest "$SpinUrl/health"
        $totalSw.Stop()

        "$Service,$i,$readyMs,$firstReqMs,$($totalSw.ElapsedMilliseconds),spin_pid=$($proc.Id)" |
            Add-Content $OutCsv -Encoding UTF8

        try {
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        } catch {}

        Start-Sleep -Milliseconds 500
    }

    if ($Service -eq "docker") {
        Stop-PortOwner -Port 8081

        docker rm -f $DockerContainer 2>$null | Out-Null

        $totalSw = [System.Diagnostics.Stopwatch]::StartNew()

        docker run -d `
            --name $DockerContainer `
            -p 8081:8081 `
            $DockerImage | Out-Null

        $readyMs = Wait-Health "$DockerUrl/health"
        $firstReqMs = Measure-OneRequest "$DockerUrl/health"
        $totalSw.Stop()

        "$Service,$i,$readyMs,$firstReqMs,$($totalSw.ElapsedMilliseconds),container=$DockerContainer" |
            Add-Content $OutCsv -Encoding UTF8

        docker rm -f $DockerContainer 2>$null | Out-Null

        Start-Sleep -Milliseconds 500
    }
}

Write-Host "Cold start results saved to: $OutCsv"