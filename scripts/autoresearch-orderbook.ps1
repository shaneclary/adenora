param(
    [string]$Label = "manual",
    [string]$EditableFile = ""
)

$ErrorActionPreference = "Stop"

function Invoke-ResearchCommand {
    param(
        [string]$Name,
        [string]$FilePath,
        [string[]]$Arguments
    )

    $quotedArgs = foreach ($argument in $Arguments) {
        if ($argument -match '\s') {
            '"' + $argument + '"'
        } else {
            $argument
        }
    }
    $commandLine = "$FilePath $($quotedArgs -join ' ')"

    $start = Get-Date
    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = & cmd /c $commandLine 2>&1
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }
    $end = Get-Date

    [pscustomobject]@{
        Name = $Name
        Command = $commandLine
        ExitCode = $exitCode
        StartedAt = $start.ToString("s")
        FinishedAt = $end.ToString("s")
        Output = ($output | Out-String).Trim()
    }
}

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$safeLabel = ($Label -replace "[^a-zA-Z0-9_-]", "_")
$reportDir = Join-Path $root "target/autoresearch"
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$reportPath = Join-Path $reportDir "$timestamp-$safeLabel.md"

$results = @(
    (Invoke-ResearchCommand -Name "orderbook-tests" -FilePath "cargo" -Arguments @("test", "-p", "adenora-orderbook")),
    (Invoke-ResearchCommand -Name "fee-tests" -FilePath "cargo" -Arguments @("test", "-p", "adenora-common", "fees")),
    (Invoke-ResearchCommand -Name "research-score" -FilePath "cargo" -Arguments @("run", "-q", "-p", "adenora-orderbook", "--bin", "research_score"))
)

$allPassed = $true
foreach ($result in $results) {
    if ($result.ExitCode -ne 0) {
        $allPassed = $false
    }
}

$researchScore = ""
$researchResult = $results | Where-Object { $_.Name -eq "research-score" } | Select-Object -First 1
if ($researchResult) {
    $scoreLine = ($researchResult.Output -split "\r?\n" | Where-Object { $_ -like "RESEARCH_SCORE=*" } | Select-Object -First 1)
    if ($scoreLine) {
        $researchScore = ($scoreLine -split "=", 2)[1].Trim()
    }
}

$status = "PASS"
if ((-not $allPassed) -or [string]::IsNullOrWhiteSpace($researchScore)) {
    $status = "FAIL"
}

$generatedAt = (Get-Date).ToString("s")

$lines = @(
    "# Autoresearch Report",
    "",
    "Status: $status",
    "Label: $Label",
    "Editable file: $EditableFile",
    "Generated at: $generatedAt",
    "Research score: $researchScore",
    "",
    "## Gates",
    ""
)

foreach ($result in $results) {
    $lines += "- $($result.Name): exit $($result.ExitCode)"
}

foreach ($result in $results) {
    $lines += @(
        "",
        "## $($result.Name)",
        "",
        "- Command: $($result.Command)",
        "- Started: $($result.StartedAt)",
        "- Finished: $($result.FinishedAt)",
        "- Exit code: $($result.ExitCode)",
        "",
        '```text',
        $result.Output,
        '```'
    )
}

Set-Content -Path $reportPath -Value $lines

Write-Host "Autoresearch report: $reportPath"
Write-Host "Status: $status"

if (-not $allPassed) {
    exit 1
}
