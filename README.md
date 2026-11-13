# RBTray-rs 🦀

Reimplementación moderna, segura y de alto rendimiento de **RBTray** escrita en **Rust**, utilizando los bindings oficiales de Microsoft ([`windows-sys`](https://crates.io/crates/windows-sys)).

Permite **minimizar cualquier ventana hacia el área de notificación (System Tray)** de Windows al hacer clic derecho sobre su botón de minimizar.

---

## 🎖️ Créditos y Reconocimientos

Este proyecto está basado e inspirado directamente en la versión original de **RBTray** (1998-2010), creada por **Nikolay Redko** y **J.D. Purcell**, distribuida bajo licencia GNU GPL v2.
- Repositorio original en SourceForge: [http://rbtray.sourceforge.net/](http://rbtray.sourceforge.net/) (espejo en GitHub: [benbuck/rbtray](https://github.com/benbuck/rbtray)).
- Agradecimiento especial a sus autores originales por la ingeniosa idea del hook de botón no-cliente (`HTMINBUTTON`).
- Esta versión en Rust preserva la filosofía ligera y sin dependencias del proyecto original, mejorándola con seguridad de memoria, colecciones dinámicas y compatibilidad moderna.

---

## 🚀 Mejoras respecto a la versión original de C++ (2010)

1. **Sin ventana de consola (`windows_subsystem`):** Corre en segundo plano de manera 100% invisible y silenciosa.
2. **Memoria y Concurrencia Segura:** Sin punteros crudos expuestos ni desbordamientos de memoria.
3. **Capacidad ilimitada de ventanas:** Reemplazo del buffer estático de 64 ventanas por colecciones dinámicas (`HashMap`).
4. **Huella mínima:** Binario nativo optimizado con LTO y stripping (`opt-level = "z"`), consumiendo < 2 MB de memoria RAM.
5. **Resiliencia ante caídas de Explorer:** Si el proceso `explorer.exe` se reinicia, todos los iconos se registran nuevamente de manera automática (`TaskbarCreated`).
6. **Restauración limpia al salir:** Si cierras RBTray-rs (o ejecutas `rbtray.exe --exit`), restaura automáticamente todas las ventanas que estuvieran ocultas en ese momento.

---

## ℹ️ ¿Por qué inicio automático de usuario en lugar de un Servicio de Windows?

En Windows (a partir de Windows Vista/7/10/11), los **Servicios de Windows corren en la "Sesión 0" (*Session 0 Isolation*)**, un entorno aislado sin interfaz gráfica de usuario por motivos de seguridad del sistema.
- Un servicio en Sesión 0 **no puede** interactuar con las ventanas del escritorio del usuario ni inyectar hooks de ratón (`WH_MOUSE`).
- Un servicio en Sesión 0 **no puede** mostrar iconos en la barra de tareas o área de notificación del usuario (`Shell_NotifyIcon`).

Por ello, el mecanismo estándar de Windows para herramientas interactivas de bandeja (como OneDrive, Discord, Steam, RBTray, etc.) es registrarse en el inicio de sesión del usuario (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`), ejecutándose silenciosamente en segundo plano.

---

## 📦 Instalación y Distribución

### Opción 1: Instalador gráfico oficial de Windows (.exe)
Dentro de la carpeta `dist/` encontrarás el instalador generado con Inno Setup:
- **`dist/RBTray-rs-Setup-v0.1.0.exe`**

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
├── crates/
│   ├── rbhook/              # DLL de inyección de hooks globales de Windows (cdylib)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs       # WH_MOUSE y WH_CALLWNDPROCRET
│   └── rbtray/              # Proceso principal en segundo plano (bin, windows_subsystem)
│       ├── Cargo.toml
│       └── src/main.rs      # Bucle de eventos, gestión de bandeja y menús
└── dist/                    # Binarios listos para usar (rbtray.exe + rbhook.dll)
```

---

## 📖 Uso

1. Haz **clic derecho** sobre el botón minimizar (`_`) de cualquier ventana para enviarla a la bandeja.
2. Para restaurar: **clic izquierdo** sobre el icono en la bandeja.
3. Para ver opciones: **clic derecho** sobre el icono en la bandeja (*Restaurar ventana*, *Cerrar ventana*, *Acerca de*, *Salir*).
4. Para salir por línea de comandos: `rbtray.exe --exit`.
