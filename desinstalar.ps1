# Script de desinstalación para RBTray-rs
$ErrorActionPreference = "SilentlyContinue"

Write-Host "=== Desinstalador de RBTray-rs ===" -ForegroundColor Cyan

$installDir = Join-Path $env:LOCALAPPDATA "Programs\RBTray-rs"
$regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$regName = "RBTray-rs"

# 1. Detener proceso
Write-Host "Deteniendo RBTray-rs..." -ForegroundColor Yellow
$proc = Get-Process -Name "rbtray" -ErrorAction SilentlyContinue
if ($proc) {
    # Intentar salida limpia con parámetro para restaurar ventanas primero
    & (Join-Path $installDir "rbtray.exe") --exit
    Start-Sleep -Milliseconds 500
    Stop-Process -Name "rbtray" -Force -ErrorAction SilentlyContinue
}

# 2. Eliminar del inicio automático de Windows
Write-Host "Eliminando del inicio automático..." -ForegroundColor Yellow
Remove-ItemProperty -Path $regPath -Name $regName -ErrorAction SilentlyContinue

# 3. Eliminar acceso directo del Menú Inicio
$programsPath = [System.IO.Path]::Combine($env:APPDATA, "Microsoft\Windows\Start Menu\Programs")
Remove-Item (Join-Path $programsPath "RBTray-rs.lnk") -Force -ErrorAction SilentlyContinue

# 4. Eliminar archivos
Write-Host "Eliminando archivos instalados..." -ForegroundColor Yellow
Remove-Item -Path $installDir -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "`n✓ RBTray-rs ha sido desinstalado por completo." -ForegroundColor Green

