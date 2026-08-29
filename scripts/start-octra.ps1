# Uruchamia Octra (mockup UI) z G:\Octra
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Exe = Join-Path $Root "src-tauri\target\release\octra.exe"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

Set-Location $Root

if (-not (Test-Path $Exe)) {
    Write-Host "Pierwsze uruchomienie - budowanie Octra (kilka minut)..." -ForegroundColor Cyan
    npm run build:app
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not (Test-Path $Exe)) {
    Write-Host "Brak octra.exe. Uruchom: npm run tauri build -- --no-bundle" -ForegroundColor Red
    exit 1
}

Start-Process -FilePath $Exe -WorkingDirectory $Root
