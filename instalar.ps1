# Script de instalación para RBTray-rs
$ErrorActionPreference = "Stop"

Write-Host "=== Instalador de RBTray-rs ===" -ForegroundColor Cyan

$installDir = Join-Path $env:LOCALAPPDATA "Programs\RBTray-rs"
$sourceDir = Join-Path $PSScriptRoot "dist"
$exePath = Join-Path $installDir "rbtray.exe"
$dllPath = Join-Path $installDir "rbhook.dll"

# 1. Verificar si existen los binarios compilados
if (-not (Test-Path (Join-Path $sourceDir "rbtray.exe"))) {
    Write-Host "Binarios no encontrados en dist/. Compilando primero..." -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot "build.ps1")
}

# 2. Detener proceso anterior si está en ejecución
$running = Get-Process -Name "rbtray" -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "Cerrando instancia previa de RBTray-rs..." -ForegroundColor Yellow
    Stop-Process -Name "rbtray" -Force
    Start-Sleep -Milliseconds 500
}

# 3. Crear directorio de instalación
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

# 4. Copiar archivos
Write-Host "Copiando archivos a $installDir..." -ForegroundColor Green
Copy-Item (Join-Path $sourceDir "rbtray.exe") -Destination $installDir -Force
try {
    Copy-Item (Join-Path $sourceDir "rbhook.dll") -Destination $installDir -Force
} catch {
    Write-Warning "rbhook.dll está retenida en memoria por Windows. Se actualizará en el próximo inicio de sesión."
}
Copy-Item (Join-Path $PSScriptRoot "README.md") -Destination $installDir -Force -ErrorAction SilentlyContinue

# 5. Configurar inicio automático con Windows (Registro de usuario)
$regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$regName = "RBTray-rs"
$regValue = "`"$exePath`""

Write-Host "Configurando inicio automático con Windows..." -ForegroundColor Green
Set-ItemProperty -Path $regPath -Name $regName -Value $regValue

# 6. Crear acceso directo en el Menú Inicio
$programsPath = [System.IO.Path]::Combine($env:APPDATA, "Microsoft\Windows\Start Menu\Programs")
$wshShell = New-Object -ComObject WScript.Shell
$shortcut = $wshShell.CreateShortcut((Join-Path $programsPath "RBTray-rs.lnk"))
$shortcut.TargetPath = $exePath
$shortcut.WorkingDirectory = $installDir
$shortcut.Description = "Minimiza cualquier ventana al System Tray con clic derecho en minimizar"
$shortcut.Save()

# 7. Iniciar la aplicación en segundo plano (sin ventana de consola)
Write-Host "Iniciando RBTray-rs en segundo plano..." -ForegroundColor Green
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $exePath
$psi.WorkingDirectory = $installDir
$psi.UseShellExecute = $true
[System.Diagnostics.Process]::Start($psi) | Out-Null

Write-Host "`n✓ ¡Instalación completada con éxito!" -ForegroundColor Cyan
Write-Host "• Ubicación: $installDir" -ForegroundColor White
Write-Host "• Ya está en ejecución en segundo plano (completamente silencioso)." -ForegroundColor White
Write-Host "• Iniciará automáticamente cada vez que inicies sesión en Windows." -ForegroundColor White
Write-Host "• Para desinstalar en cualquier momento, ejecuta: desinstalar.ps1" -ForegroundColor Gray
