# Builds the recall comparison from the benchmark reports in results\.
#
# Outputs (all derived from the reports, nothing typed by hand):
#   results\recall_by_config.csv   one row per configuration, recall per data set
#   results\recall_by_config.svg   grouped bar chart (higher bar = higher recall)
#   results\recall_table.md        Markdown table, the highest value of each column in bold
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File verification\recall\make_recall_report.ps1

$ErrorActionPreference = "Stop"
$results = Join-Path $PSScriptRoot "results"

# Configuration name, report file, in the order shown in the outputs.
$configs = @(
    @{ name = "Default (previous), efSearch=64";          file = "exploration\sweep_efs_64.md" },
    @{ name = "Default (previous), efSearch=128";         file = "exploration\sweep_efs_128.md" },
    @{ name = "Default (previous), efSearch=256";         file = "exploration\sweep_efs_256.md" },
    @{ name = "Default (previous), efSearch=512";         file = "exploration\sweep_efs_512.md" },
    @{ name = "Heuristic, M0=16, efSearch=64";            file = "exploration\heur_efs64.md" },
    @{ name = "Heuristic, M0=16, efSearch=256";           file = "exploration\heur_efs256.md" },
    @{ name = "Heuristic, no fill, M0=16";                file = "exploration\g_heur_nokeep_m16.md" },
    @{ name = "Heuristic, M=8, M0=16";                    file = "exploration\g_heur_m8_m0x2.md" },
    @{ name = "M0=32, no heuristic, efSearch=64";         file = "exploration\m0x2_efs64.md" },
    @{ name = "Heuristic, no fill, M0=32";                file = "exploration\g_heur_nokeep_m32.md" },
    @{ name = "Heuristic, M0=32, efConstruction=100";     file = "exploration\g_heur_m0x2_efc100.md" },
    @{ name = "Heuristic, M0=32, efConstruction=400";     file = "exploration\g_heur_m0x2_efc400.md" },
    @{ name = "Heuristic, M=32, M0=64";                   file = "exploration\g_heur_m32_m0x2.md" },
    @{ name = "Heuristic, M0=32, efSearch=64 (reference)"; file = "earlier\earlier_01.md" }
)

$columns = @(
    @{ key = "uniform_1000";     shape = "uniform";   n = "1000" },
    @{ key = "clustered_1000";   shape = "clustered"; n = "1000" },
    @{ key = "uniform_10000";    shape = "uniform";   n = "10000" },
    @{ key = "clustered_10000";  shape = "clustered"; n = "10000" }
)

# Read recall for every configuration from its report.
$rows = foreach ($c in $configs) {
    $path = Join-Path $results $c.file
    $values = @{}
    foreach ($line in Get-Content $path) {
        if ($line -match '^\| (uniform|clustered) \| (\d+) \|') {
            $shape = $Matches[1]; $n = $Matches[2]
            $cells = $line.Split('|')
            $values["${shape}_$n"] = [double]$cells[5].Trim()
        }
    }
    [pscustomobject]@{
        configuration       = $c.name
        "uniform_1000"      = $values["uniform_1000"]
        "clustered_1000"    = $values["clustered_1000"]
        "uniform_10000"     = $values["uniform_10000"]
        "clustered_10000"   = $values["clustered_10000"]
        source              = $c.file
    }
}

$csv = Join-Path $results "recall_by_config.csv"
$rows | Export-Csv -Path $csv -NoTypeInformation -Encoding UTF8

# Highest value per column (ties: all tied values count as highest).
$max = @{}
foreach ($col in $columns) {
    $max[$col.key] = ($rows | Measure-Object -Property $col.key -Maximum).Maximum
}

# Markdown table with the highest value of each column in bold.
$md = New-Object System.Collections.Generic.List[string]
$md.Add("| Configuration | uniform 1,000 | clustered 1,000 | uniform 10,000 | clustered 10,000 |")
$md.Add("| :--- | ---: | ---: | ---: | ---: |")
foreach ($r in $rows) {
    $cells = foreach ($col in $columns) {
        $v = $r.($col.key)
        $text = "{0:N4}" -f $v
        if ($v -eq $max[$col.key]) { "**$text**" } else { $text }
    }
    $md.Add("| $($r.configuration) | " + ($cells -join " | ") + " |")
}
$mdPath = Join-Path $results "recall_table.md"
$md | Set-Content -Path $mdPath -Encoding UTF8

# Grouped bar chart as SVG (horizontal bars, one group per configuration).
$width = 1080; $leftPad = 330; $rightPad = 150; $topPad = 70
$groupH = 46; $barH = 9; $barGap = 2
$chartW = $width - $leftPad - $rightPad
$height = $topPad + $rows.Count * $groupH + 60
$colors = @{ "uniform_1000" = "#8ecae6"; "clustered_1000" = "#219ebc"; "uniform_10000" = "#ffb703"; "clustered_10000" = "#fb8500" }
$svg = New-Object System.Collections.Generic.List[string]
$svg.Add("<svg xmlns=""http://www.w3.org/2000/svg"" width=""$width"" height=""$height"" viewBox=""0 0 $width $height"" font-family=""Arial, Helvetica, sans-serif"" font-size=""12"">")
$svg.Add("<rect width=""100%"" height=""100%"" fill=""#ffffff""/>")
$svg.Add("<text x=""$leftPad"" y=""28"" font-size=""16"" font-weight=""bold"">Recall@10 by configuration (higher is better)</text>")
# Legend
$lx = $leftPad
foreach ($col in $columns) {
    $label = "{0} {1}" -f $col.shape, ("{0:N0}" -f [int]$col.n)
    $svg.Add("<rect x=""$lx"" y=""40"" width=""12"" height=""12"" fill=""$($colors[$col.key])""/>")
    $svg.Add("<text x=""$($lx + 17)"" y=""51"" >$label</text>")
    $lx += 170
}
# Scale lines at 0, 0.5, 1.0
foreach ($t in @(0, 0.5, 1.0)) {
    $x = $leftPad + $t * $chartW
    $svg.Add("<line x1=""$x"" y1=""$topPad"" x2=""$x"" y2=""$($height - 50)"" stroke=""#dddddd""/>")
    $svg.Add("<text x=""$x"" y=""$($height - 34)"" text-anchor=""middle"" fill=""#555555"">$t</text>")
}
for ($i = 0; $i -lt $rows.Count; $i++) {
    $r = $rows[$i]
    $gy = $topPad + $i * $groupH
    $svg.Add("<text x=""$($leftPad - 10)"" y=""$($gy + 22)"" text-anchor=""end"">$([System.Security.SecurityElement]::Escape($r.configuration))</text>")
    for ($k = 0; $k -lt $columns.Count; $k++) {
        $col = $columns[$k]
        $v = [double]$r.($col.key)
        $y = $gy + 4 + $k * ($barH + $barGap)
        $w = [math]::Max(1, $v * $chartW)
        $isMax = ($v -eq $max[$col.key])
        $stroke = if ($isMax) { " stroke=""#000000"" stroke-width=""1.5""" } else { "" }
        $svg.Add("<rect x=""$leftPad"" y=""$y"" width=""$([math]::Round($w,1))"" height=""$barH"" fill=""$($colors[$col.key])""$stroke/>")
        $label = "{0:N4}" -f $v
        if ($isMax) { $label = "$label (highest)" }
        $svg.Add("<text x=""$($leftPad + $w + 4)"" y=""$($y + $barH - 1)"" font-size=""10"">$label</text>")
    }
}
$svg.Add("</svg>")
$svgPath = Join-Path $results "recall_by_config.svg"
$svg | Set-Content -Path $svgPath -Encoding UTF8

# Before / after chart: previous default vs current default, per data set.
$prevRow = $rows | Where-Object { $_.configuration -eq "Default (previous), efSearch=64" }
$currRow = $rows | Where-Object { $_.configuration -eq "Heuristic, M0=32, efSearch=64 (reference)" }
$groupDefs = @(
    @{ label = "uniform, 1,000 vectors";    key = "uniform_1000" },
    @{ label = "clustered, 1,000 vectors";  key = "clustered_1000" },
    @{ label = "uniform, 10,000 vectors";   key = "uniform_10000" },
    @{ label = "clustered, 10,000 vectors"; key = "clustered_10000" }
)
$baW = 1000; $baLeft = 250; $baBarMax = 640; $baTop = 92; $baGroup = 84
$baH = $baTop + $groupDefs.Count * $baGroup + 24
$ba = New-Object System.Collections.Generic.List[string]
$ba.Add("<svg xmlns=""http://www.w3.org/2000/svg"" width=""$baW"" height=""$baH"" viewBox=""0 0 $baW $baH"" font-family=""Arial, Helvetica, sans-serif"" font-size=""13"">")
$ba.Add("<rect width=""100%"" height=""100%"" fill=""#ffffff""/>")
$ba.Add("<text x=""20"" y=""30"" font-size=""18"" font-weight=""bold"">Recall@10: previous default vs current default</text>")
$ba.Add("<text x=""20"" y=""52"" fill=""#555555"">Higher is better. Synthetic data, 384 dimensions, 1,000 queries.</text>")
$ba.Add("<rect x=""20"" y=""66"" width=""14"" height=""14"" fill=""#9e9e9e""/><text x=""40"" y=""78"">Previous default (nearest-M, M0 = 16)</text>")
$ba.Add("<rect x=""340"" y=""66"" width=""14"" height=""14"" fill=""#2a9d8f""/><text x=""360"" y=""78"">Current default (heuristic, M0 = 32)</text>")
for ($i = 0; $i -lt $groupDefs.Count; $i++) {
    $g = $groupDefs[$i]
    $y0 = $baTop + $i * $baGroup
    $pv = [double]$prevRow.($g.key)
    $cv = [double]$currRow.($g.key)
    $ba.Add("<text x=""20"" y=""$($y0 + 24)"" font-size=""14"" font-weight=""bold"">$($g.label)</text>")
    foreach ($bar in @(@{ v = $pv; fill = "#9e9e9e"; dy = 0 }, @{ v = $cv; fill = "#2a9d8f"; dy = 30 })) {
        $y = $y0 + $bar.dy
        $w = [math]::Round($bar.v * $baBarMax, 1)
        $ba.Add("<rect x=""$baLeft"" y=""$y"" width=""$w"" height=""22"" fill=""$($bar.fill)""/>")
        $ba.Add("<text x=""$($baLeft + $w + 6)"" y=""$($y + 16)"">" + ("{0:N4}" -f $bar.v) + "</text>")
    }
}
$ba.Add("</svg>")
$baPath = Join-Path $results "recall_before_after.svg"
$ba | Set-Content -Path $baPath -Encoding UTF8

Write-Host "wrote $csv"
Write-Host "wrote $mdPath"
Write-Host "wrote $svgPath"
