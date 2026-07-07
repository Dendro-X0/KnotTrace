# KnotTrace - set TAURI_SIGNING_PRIVATE_KEY on GitHub (Windows)
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$KeyPath = Join-Path $Root "apps\desktop\src-tauri\.updater\knottrace.key"
$PubPath = "$KeyPath.pub"
$Repo = if ($env:GITHUB_REPOSITORY) { $env:GITHUB_REPOSITORY } else { "Dendro-X0/KnotTrace" }

if (-not (Test-Path $KeyPath)) {
    Write-Host "Private key not found. Run: bash ./scripts/generate-updater-keys.sh" -ForegroundColor Red
    exit 1
}

Write-Host "KnotTrace updater signing - GitHub secret setup"
Write-Host "=============================================="
Write-Host ""
Write-Host "Public key (must match tauri.conf.json):"
Get-Content $PubPath
Write-Host ""

$gh = Get-Command gh -ErrorAction SilentlyContinue
if ($gh) {
    gh auth status 2>$null
    if ($LASTEXITCODE -eq 0) {
        Get-Content $KeyPath -Raw | gh secret set TAURI_SIGNING_PRIVATE_KEY --repo $Repo
        gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo $Repo --body ""
        Write-Host ""
        Write-Host "Done. Re-run: Actions -> Release -> Run workflow -> v1.1.1" -ForegroundColor Green
        exit 0
    }
}

Write-Host "Manual setup:"
Write-Host "1. Open https://github.com/$Repo/settings/secrets/actions"
Write-Host "2. New secret: TAURI_SIGNING_PRIVATE_KEY"
Write-Host "   Value: copy entire file at:"
Write-Host "   $KeyPath"
Write-Host "3. New secret: TAURI_SIGNING_PRIVATE_KEY_PASSWORD (empty)"
Write-Host "4. Re-run release workflow for tag v1.1.1"
Write-Host ""
Write-Host "Copy private key to clipboard:"
$clipCmd = "Get-Content -Raw `"$KeyPath`" | Set-Clipboard"
Write-Host "  $clipCmd"
