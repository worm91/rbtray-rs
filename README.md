# RBTray-rs 🦀

Reimplementación moderna, segura y de alto rendimiento de **RBTray** escrita en **Rust**, utilizando los bindings oficiales de Microsoft ([`windows-sys`](https://crates.io/crates/windows-sys)).

Permite **minimizar cualquier ventana hacia el área de notificación (System Tray)** de Windows al hacer clic derecho sobre su botón de minimizar o mediante atajos de teclado globales.

---

## 🎖️ Créditos y Reconocimientos

Este proyecto está basado e inspirado directamente en la versión original de **RBTray** (1998-2010), creada por **Nikolay Redko** y **J.D. Purcell**, distribuida bajo licencia GNU GPL v2.
- Repositorio original en SourceForge: [http://rbtray.sourceforge.net/](http://rbtray.sourceforge.net/) (espejo en GitHub: [benbuck/rbtray](https://github.com/benbuck/rbtray)).
- Agradecimiento especial a sus autores originales por la ingeniosa idea del hook de botón no-cliente (`HTMINBUTTON`).
- Esta versión en Rust preserva la filosofía ligera y sin dependencias del proyecto original, mejorándola con seguridad de memoria, colecciones dinámicas y compatibilidad moderna.

---

## 🚀 Mejoras y Novedades (v0.2.0)

1. **Soporte para aplicaciones modernas (Electron, CSD, Chrome, Spotify, Discord, etc.):**
   - No solo detecta `HTMINBUTTON` estándar en aplicaciones Win32 clásicas, sino también ventanas con barras de título personalizadas y eventos de área cliente (`WM_RBUTTONUP` con hit-testing heurístico).
2. **Atajos Globales de Teclado (*Global Hotkeys*):**
   - **`Win + Alt + Flecha Abajo`**: Minimiza instantáneamente la ventana activa al System Tray.
   - **`Win + Alt + Flecha Arriba`**: Restaura la última ventana minimizada.
   - **`Win + Alt + R`**: Restaura **todas** las ventanas minimizadas simultáneamente.
3. **Modo Oscuro Nativo en Menús (Windows 10/11):**
   - Integración con las APIs de UxTheme para que el menú contextual se dibuje en modo oscuro o claro según la preferencia del sistema operativo.
4. **Menú contextual extendido:**
   - Opciones para *"Restaurar todas las ventanas"* y *"Cerrar todas las ventanas"*.
5. **Lista de Exclusión (*Blacklist*) y Configuración:**
   - Archivo `config.json` para definir procesos que no deben minimizarse (ej. Administrador de Tareas, procesos del sistema).
6. **Sincronización Dinámica de Títulos y Limpieza de Huérfanos:**
   - Monitoreo en tiempo real: si una ventana minimizada cambia de título (ej. progreso de descarga o nuevos mensajes), el tooltip en la bandeja se actualiza automáticamente. Si la aplicación finaliza en segundo plano, su icono se limpia sin dejar iconos fantasma.
7. **Sin ventana de consola (`windows_subsystem = "windows"`):**
   - Ejecución 100% silenciosa en segundo plano, consumiendo menos de 2 MB de memoria RAM.

---

## 📦 Instalación y Distribución

### Opción 1: Instalador gráfico oficial de Windows (.exe)
Dentro de la carpeta `dist/` o en las Releases de GitHub encontrarás el instalador generado con Inno Setup:
- **`dist/RBTray-rs-Setup-v0.2.0.exe`**

Este instalador:
1. Incluye asistente gráfico paso a paso en español e inglés.
2. Muestra la licencia GNU GPL v2.
3. Te permite elegir si arrancar con Windows y crear acceso directo en el escritorio.
4. Se registra en **"Aplicaciones instaladas" de Windows (Configuración / Panel de Control)** para desinstalarlo en cualquier momento.

### Opción 2: Instalación rápida por PowerShell
Ejecuta en PowerShell:
```powershell
.\instalar.ps1
```

### Desinstalación por script
```powershell
.\desinstalar.ps1
```

---

## 📁 Estructura del Proyecto

```text
rbtray-rs/
├── Cargo.toml               # Configuración del workspace y optimizaciones release
├── build.ps1                # Script automatizado de compilación y empaquetado
├── instalar.ps1             # Instalador e inicio automático
├── desinstalar.ps1          # Desinstalador limpio
├── installer.iss            # Script de Inno Setup para generar el Setup .exe
├── assets/                  # Iconos y licencia
├── crates/
│   ├── rbhook/              # DLL de inyección de hooks globales de Windows (cdylib)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs       # WH_MOUSE (con soporte CSD/Electron) y WH_CALLWNDPROCRET
│   └── rbtray/              # Proceso principal en segundo plano (bin, windows_subsystem)
│       ├── Cargo.toml
│       └── src/main.rs      # Bucle de eventos, atajos globales, modo oscuro y bandeja
└── dist/                    # Binarios listos para usar e instalador Setup .exe
```

---

## 📖 Uso

- **Con ratón:** Haz **clic derecho** sobre el botón minimizar (`_`) de cualquier ventana (clásica o Electron/CSD).
- **Con teclado:** Pulsa **`Win + Alt + Flecha Abajo`** en cualquier momento para minimizar la ventana en primer plano.
- **Para restaurar:**
  - **Clic izquierdo** sobre su icono en la bandeja.
  - O pulsa **`Win + Alt + Flecha Arriba`** para restaurar la última ventana.
  - O pulsa **`Win + Alt + R`** para restaurar todas.
- **Menú contextual:** **Clic derecho** sobre cualquier icono en la bandeja para ver opciones extendidas.
- **Salir:** Clic derecho en el icono -> *Salir de RBTray-rs* (o ejecutando `rbtray.exe --exit`).
