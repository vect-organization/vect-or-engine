# Reproduces the recall measurement for the reference HNSW configuration.
#
# Reference configuration:
#   heuristic neighbour selection, rejected candidates kept (default),
#   M = 16, layer-0 cap M0 = 32, efConstruction = 200, efSearch = 64
#
# Usage (from the repository root or from this folder):
#   powershell -ExecutionPolicy Bypass -File verification\recall\reproduce.ps1 -Runs 2
#
# Output:
#   verification\recall\results\runNN.md          full benchmark report of each run
#   verification\recall\results\recall_summary.csv recall and p50 per run, shape and N

param(
    [int]$Runs = 2,
    [string]$Sizes = "1000,10000",
    [int]$Queries = 1000
)

$ErrorActionPreference = "Stop"
$repo = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$results = Join-Path $PSScriptRoot "results"
New-Item -ItemType Directory -Force -Path $results | Out-Null

Push-Location $repo
try {
    for ($i = 1; $i -le $Runs; $i++) {
        $file = Join-Path $results ("run{0:D2}.md" -f $i)
        Write-Host "run $i -> $file"
        & cargo run -q -j 1 --release --example bench -- `
            --sizes $Sizes --queries $Queries `
            --heuristic --m0 32 --ef-search 64 `
            --out $file | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "bench failed in run $i" }
    }
}
finally {
    Pop-Location
}

# Collect recall and p50 from every run report (columns: shape | N | params | build | recall | p50 ...).
$rows = foreach ($f in Get-ChildItem $results -Filter "run*.md" | Sort-Object Name) {
    Select-String -Path $f.FullName -Pattern '^\| (uniform|clustered) \|' | ForEach-Object {
        $c = $_.Line.Split('|')
        [pscustomobject]@{
            run     = $f.BaseName
            shape   = $c[1].Trim()
            N       = [int]$c[2].Trim()
            recall  = [double]$c[5].Trim()
            p50_us  = [double]$c[6].Trim()
        }
    }
}

$csv = Join-Path $results "recall_summary.csv"
$rows | Export-Csv -Path $csv -NoTypeInformation -Encoding UTF8
$rows | Format-Table -AutoSize
Write-Host "summary written to $csv"
