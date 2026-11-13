# RBTray-rs 🦀

> [🇺🇸 Read in English](README.md)

Reimplementación moderna, segura y de alto rendimiento de **RBTray** escrita en **Rust**, utilizando los bindings oficiales de Microsoft ([`windows-sys`](https://crates.io/crates/windows-sys)).

Permite **minimizar cualquier ventana hacia el área de notificación (System Tray)** de Windows al hacer clic derecho sobre su botón de minimizar o mediante atajos globales de teclado.

---

## 🎖️ Créditos y Reconocimientos

- **Desarrollo y Reimplementación Moderna (RBTray-rs):**
  - **Luis Alberto Garcia (@worm91)** — Arquitectura en Rust, soporte para aplicaciones modernas (Electron / CSD), atajos globales de teclado (`Ctrl+Win`), modo oscuro nativo, sincronización dinámica de títulos, lista de exclusión e instalador oficial.

- **Autores de la Idea Original y Versión Clásica en C++ (1998–2010):**
  - **Nikolay Redko** y **J.D. Purcell** — Por la ingeniosa técnica original del hook Win32 sobre el botón de minimizar (`HTMINBUTTON`).
  - Repositorio histórico en SourceForge: [http://rbtray.sourceforge.net/](http://rbtray.sourceforge.net/) (espejo en GitHub: [benbuck/rbtray](https://github.com/benbuck/rbtray)).

- **Licencia:** Distribuido bajo los términos de software libre de la Licencia Pública General GNU v2 (**GNU GPL v2**).

---

## 🚀 Mejoras y Novedades (v0.2.0)

1. **Soporte para aplicaciones modernas (Electron, CSD, Chrome, Spotify, Discord, VS Code, etc.):**
   - Intercepta clics sobre el botón de minimizar incluso en aplicaciones sin marco tradicional de Windows mediante hit-testing heurístico.
2. **Atajos Globales de Teclado (*Global Hotkeys*):**
   - **`Ctrl + Win + Flecha Abajo`**: Minimiza instantáneamente la ventana activa al System Tray.
   - **`Ctrl + Win + Flecha Arriba`**: Restaura la última ventana minimizada.
   - **`Ctrl + Win + 0`**: Restaura **todas** las ventanas minimizadas simultáneamente.
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

## 📖 Uso

- **Con ratón:** Haz **clic derecho** sobre el botón minimizar (`_`) de cualquier ventana (clásica o Electron/CSD).
- **Con teclado:**
  - Pulsa **`Ctrl + Win + Flecha Abajo`** para minimizar la ventana activa al tray.
  - Pulsa **`Ctrl + Win + Flecha Arriba`** para restaurar la última ventana minimizada.
  - Pulsa **`Ctrl + Win + 0`** para restaurar todas las ventanas minimizadas.
- **Menú contextual:** **Clic derecho** sobre cualquier icono en la bandeja para ver opciones extendidas.
- **Salir:** Clic derecho en el icono -> *Salir de RBTray-rs* (o ejecutando `rbtray.exe --exit`).
