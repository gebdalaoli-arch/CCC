Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent -Path $MyInvocation.MyCommand.Definition
Set-Location $projectRoot

Write-Host "[build] npm run tauri:build:windows"
npm run tauri:build:windows

if ($LASTEXITCODE -ne 0) {
    throw "Windows build failed."
}

$bundle = Join-Path $projectRoot "src-tauri\target\release\bundle\nsis\Codex Launcher_0.1.0_x64-setup.exe"
if (Test-Path -LiteralPath $bundle) {
    Write-Host "[done] $bundle"
} else {
    Write-Warning "Build completed, but installer path was not found: $bundle"
}
