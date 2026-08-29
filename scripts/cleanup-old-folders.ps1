# Usuwa stare foldery Lumen po zamknieciu Cursora i zakonczeniu octra.exe
# Uruchom: powershell -ExecutionPolicy Bypass -File scripts/cleanup-old-folders.ps1
$ErrorActionPreference = "Continue"
taskkill /IM octra.exe /F 2>$null | Out-Null
taskkill /IM lumen.exe /F 2>$null | Out-Null
Start-Sleep -Seconds 2

$old = @(
    "G:\Octra Launcher",
    "G:\Mój Launcher Minecraft"
)

foreach ($p in $old) {
    if (-not (Test-Path -LiteralPath $p)) {
        Write-Host "Juz brak: $p" -ForegroundColor DarkGray
        continue
    }
    Write-Host "Usuwam: $p ..."
    Remove-Item -LiteralPath $p -Recurse -Force -ErrorAction SilentlyContinue
    if (Test-Path -LiteralPath $p) {
        Write-Host "  ZABLOKOWANY - zamknij Cursor/okna z tego folderu i sprobuj ponownie." -ForegroundColor Yellow
    } else {
        Write-Host "  OK" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "Jedyny projekt: G:\Octra" -ForegroundColor Cyan
