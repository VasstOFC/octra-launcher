# Octra — tryb deweloperski (hot reload)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
Set-Location $Root
npm run tauri dev
