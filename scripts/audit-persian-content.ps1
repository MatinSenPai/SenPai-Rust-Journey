param(
    [string]$Root = (Split-Path -Parent $PSScriptRoot),
    [switch]$RequireComplete
)

$ErrorActionPreference = 'Stop'
$areas = @(
    'phase0-setup',
    'phase1-fundamentals',
    'phase2-intermediate',
    'phase3-backend-foundations',
    'phase4-backend-advanced',
    'phase5-system-design-mastery',
    'capstone-taskforge',
    'side-quests',
    'docs'
)

$canonical = foreach ($area in $areas) {
    $areaPath = Join-Path $Root $area
    if (Test-Path -LiteralPath $areaPath) {
        Get-ChildItem -LiteralPath $areaPath -Recurse -File -Filter '*.md' |
            Where-Object { $_.Name -notlike '*.fa.md' }
    }
}
$canonical = @(
    Get-Item -LiteralPath (Join-Path $Root 'README.md')
    Get-Item -LiteralPath (Join-Path $Root 'PROGRESS.md')
) + @($canonical)

$missing = foreach ($file in $canonical) {
    $companion = Join-Path $file.DirectoryName ($file.BaseName + '.fa.md')
    if (-not (Test-Path -LiteralPath $companion)) {
        $file.FullName.Substring($Root.Length + 1)
    }
}

$invalidVisuals = @()
$knownKinds = @('ownership', 'borrowing', 'lifetime', 'result', 'async', 'queue', 'database', 'network', 'concurrency', 'roadmap', 'concept')
Get-ChildItem -LiteralPath $Root -Recurse -File -Filter '*.fa.md' |
    Where-Object { $_.FullName -notlike '*\target\*' } |
    ForEach-Object {
        $inside = $false
        foreach ($line in Get-Content -LiteralPath $_.FullName) {
            if ($line -eq '```senpai-visual') {
                $inside = $true
                continue
            }
            if ($inside) {
                try {
                    $spec = $line | ConvertFrom-Json
                    if ($knownKinds -notcontains $spec.kind) { throw "unknown kind $($spec.kind)" }
                } catch {
                    $invalidVisuals += "$($_.FullName.Substring($Root.Length + 1)): $line"
                }
                $inside = $false
            }
        }
    }

$font = Join-Path $Root 'web-ui\assets\Vazirmatn-Variable.woff2'
$css = Join-Path $Root 'web-ui\src\style.rs'
$fontBytes = if (Test-Path -LiteralPath $font) { (Get-Item -LiteralPath $font).Length } else { 0 }
$cssBytes = if (Test-Path -LiteralPath $css) { (Get-Item -LiteralPath $css).Length } else { 0 }

Write-Host "Canonical Markdown: $($canonical.Count)"
Write-Host "Persian companions: $($canonical.Count - $missing.Count)"
Write-Host "Missing companions: $($missing.Count)"
Write-Host "Invalid visuals: $($invalidVisuals.Count)"
Write-Host "Vazirmatn bytes: $fontBytes / 153600"
Write-Host "CSS source bytes: $cssBytes / 51200"

if ($missing.Count -gt 0) {
    Write-Host "`nMissing:"
    $missing | ForEach-Object { Write-Host "  $_" }
}
if ($invalidVisuals.Count -gt 0) {
    Write-Host "`nInvalid visuals:"
    $invalidVisuals | ForEach-Object { Write-Host "  $_" }
}

$failed = $invalidVisuals.Count -gt 0 -or $fontBytes -gt 153600 -or $cssBytes -gt 51200
if ($RequireComplete -and $missing.Count -gt 0) { $failed = $true }
if ($failed) { exit 1 }
