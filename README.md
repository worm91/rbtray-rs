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
