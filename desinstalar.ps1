# Script de desinstalación para RBTray-rs
$ErrorActionPreference = "SilentlyContinue"

Write-Host "=== Desinstalador de RBTray-rs ===" -ForegroundColor Cyan

$installDir = Join-Path $env:LOCALAPPDATA "Programs\RBTray-rs"
$regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$regName = "RBTray-rs"

# 1. Detener proceso rbtray
Write-Host "Cerrando RBTray-rs y restaurando ventanas..." -ForegroundColor Yellow
$proc = Get-Process -Name "rbtray" -ErrorAction SilentlyContinue
if ($proc) {
    # Intentar salida limpia para restaurar ventanas primero
    if (Test-Path (Join-Path $installDir "rbtray.exe")) {
        & (Join-Path $installDir "rbtray.exe") --exit
    }
    Start-Sleep -Milliseconds 800
    Stop-Process -Name "rbtray" -Force -ErrorAction SilentlyContinue
}

# 2. Eliminar del inicio automático de Windows
Write-Host "Eliminando del inicio automático..." -ForegroundColor Yellow
Remove-ItemProperty -Path $regPath -Name $regName -ErrorAction SilentlyContinue

# 3. Eliminar acceso directo del Menú Inicio y Escritorio
$programsPath = [System.IO.Path]::Combine($env:APPDATA, "Microsoft\Windows\Start Menu\Programs")
Remove-Item (Join-Path $programsPath "RBTray-rs.lnk") -Force -ErrorAction SilentlyContinue
$desktopPath = [System.Environment]::GetFolderPath("Desktop")
Remove-Item (Join-Path $desktopPath "RBTray-rs.lnk") -Force -ErrorAction SilentlyContinue

# 4. Eliminar archivos generados y de configuración
Write-Host "Eliminando archivos y configuración..." -ForegroundColor Yellow
Remove-Item (Join-Path $installDir "config.json") -Force -ErrorAction SilentlyContinue
Remove-Item (Join-Path $installDir "*.log") -Force -ErrorAction SilentlyContinue
Remove-Item (Join-Path $installDir "rbtray.exe") -Force -ErrorAction SilentlyContinue
Remove-Item (Join-Path $installDir "app.ico") -Force -ErrorAction SilentlyContinue
Remove-Item (Join-Path $installDir "README*.md") -Force -ErrorAction SilentlyContinue

# 5. Intentar eliminar rbhook.dll
$dllPath = Join-Path $installDir "rbhook.dll"
if (Test-Path $dllPath) {
    Remove-Item $dllPath -Force -ErrorAction SilentlyContinue
    if (Test-Path $dllPath) {
        Write-Warning "rbhook.dll está retenida en memoria por otros procesos activos. Se programó su eliminación automática al reiniciar/cerrar sesión."
        Add-Type @"
        using System;
        using System.Runtime.InteropServices;
        public class FileOps {
            [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
            public static extern bool MoveFileEx(string lpExistingFileName, string lpNewFileName, int dwFlags);
        }
"@
        [FileOps]::MoveFileEx($dllPath, $null, 4) # MOVEFILE_DELAY_UNTIL_REBOOT = 4
    }
}

# 6. Eliminar carpeta del programa si quedó vacía
Remove-Item -Path $installDir -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "`n✓ RBTray-rs ha sido desinstalado por completo." -ForegroundColor Green
