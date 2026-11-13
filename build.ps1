# Script de compilación y empaquetado para RBTray-rs
param(
    [switch]$Release = $true
)

$ErrorActionPreference = "Stop"

Write-Host "=== RBTray-rs Build Script ===" -ForegroundColor Cyan

# 1. Asegurar PATH de Cargo
if (Test-Path "$env:USERPROFILE\.cargo\bin") {
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
}

$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoCmd) {
    Write-Warning "Cargo / Rust no esta detectado en tu PATH."
    Write-Host "Puedes instalar Rust ejecutando:" -ForegroundColor Yellow
    Write-Host "  winget install Rustlang.Rustup" -ForegroundColor White
    exit 1
}

# 2. Compilar
$modeStr = if ($Release) { "Release" } else { "Debug" }
$config = if ($Release) { "--release" } else { "" }
Write-Host "Compilando workspace en modo $modeStr..." -ForegroundColor Green

cargo build $config

if ($LASTEXITCODE -ne 0) {
    Write-Error "La compilacion fallo."
    exit $LASTEXITCODE
}

# 3. Empaquetar artefactos en dist/
$targetSub = if ($Release) { "release" } else { "debug" }
$targetDir = Join-Path $PSScriptRoot "target\$targetSub"
$distDir = Join-Path $PSScriptRoot "dist"

New-Item -ItemType Directory -Force -Path $distDir | Out-Null

Copy-Item (Join-Path $targetDir "rbtray.exe") -Destination $distDir -Force
try {
    Copy-Item (Join-Path $targetDir "rbhook.dll") -Destination $distDir -Force
} catch {
    Write-Warning "rbhook.dll esta en uso por procesos activos del sistema. Se mantuvo la version existente en dist/."
}

Write-Host "`nCompilacion exitosa. Archivos generados en:" -ForegroundColor Green
Write-Host "  $distDir\rbtray.exe" -ForegroundColor White
Write-Host "  $distDir\rbhook.dll" -ForegroundColor White

# 4. Compilar instalador de Windows (Inno Setup) si esta disponible
$isccCandidates = @(
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe"
)

$isccPath = $isccCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if ($isccPath) {
    Write-Host "`nGenerando instalador de Windows con Inno Setup..." -ForegroundColor Cyan
    & $isccPath (Join-Path $PSScriptRoot "installer.iss")
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`n[OK] Instalador de Windows creado con exito:" -ForegroundColor Green
        Write-Host "  $distDir\RBTray-rs-Setup-v0.2.0.exe" -ForegroundColor White
    }
} else {
    Write-Host "`nInno Setup no detectado. Para generar el instalador, ejecuta installer.iss." -ForegroundColor Gray
}
