# Skroty Octra na pulpicie: Dev (tauri dev) + Gra (release exe)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Desktop = [Environment]::GetFolderPath("Desktop")
$Icon = Join-Path $Root "src-tauri\icons\icon.ico"
$Wsh = New-Object -ComObject WScript.Shell

function New-Shortcut($Name, $Script) {
    $path = Join-Path $Desktop "$Name.lnk"
    $sc = $Wsh.CreateShortcut($path)
    $sc.TargetPath = "powershell.exe"
    $sc.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$Script`""
    $sc.WorkingDirectory = $Root
    if (Test-Path $Icon) { $sc.IconLocation = $Icon }
    $sc.Description = "Octra launcher Minecraft"
    $sc.Save()
    Write-Host "Skrot: $path" -ForegroundColor Green
}

New-Shortcut "Octra" (Join-Path $Root "scripts\start-octra.ps1")
New-Shortcut "Octra Dev" (Join-Path $Root "scripts\dev.ps1")
