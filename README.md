# RBTray-rs 🦀

> [🇪🇸 Leer en Español](README.es.md)

A modern, memory-safe, and high-performance reimplementation of **RBTray** written in **Rust**, utilizing Microsoft's official Win32 bindings ([`windows-sys`](https://crates.io/crates/windows-sys)).

It allows you to **minimize almost any application window directly to the Windows Notification Area (System Tray)** by right-clicking its minimize button or using global keyboard shortcuts.

---

## 🎖️ Credits & Attribution

- **Modern Reimplementation & Architecture (RBTray-rs):**
  - **Luis Alberto Garcia (@worm91)** — Rust architecture, modern application support (Electron / CSD), global keyboard shortcuts (`Ctrl+Win`), native dark mode menus, dynamic title synchronization, process exclusion list, and official Windows installer.

- **Original Authors & Concept (C++, 1998–2010):**
  - **Nikolay Redko** and **J.D. Purcell** — For the brilliant original Win32 non-client mouse hook technique (`HTMINBUTTON`).
  - Historical repository on SourceForge: [http://rbtray.sourceforge.net/](http://rbtray.sourceforge.net/) (GitHub mirror: [benbuck/rbtray](https://github.com/benbuck/rbtray)).

- **License:** Distributed under the terms of the GNU General Public License v2 (**GNU GPL v2**).

---

## 🚀 Features & Improvements (v0.2.0)

1. **Modern App Support (Electron, Chrome, Discord, Spotify, VS Code, Slack, CSD):**
   - Intercepts minimize button clicks even on frameless or client-side decorated windows using heuristic geometry and hit-testing.
2. **Global Keyboard Shortcuts (*Ergonomic & Conflict-Free*):**
   - **`Ctrl + Win + Down`**: Instantly minimizes the active foreground window to the tray.
   - **`Ctrl + Win + Up`**: Restores the most recently minimized window (LIFO stack).
   - **`Ctrl + Win + 0`**: Restores **all** minimized windows at once.
3. **Native Dark Mode Menus (Windows 10 / 11):**
   - Seamlessly integrates with Windows theme settings via UxTheme APIs; popup menus automatically match your system's Dark or Light theme.
4. **Extended Context Menu:**
   - Single-window management: *"Restore Window"* and *"Close Window"*.
   - Batch management: *"Restore All Windows"* and *"Close All Windows"*.
5. **Process Exclusion List (*Blacklist*):**
   - Configurable `config.json` file in `%LOCALAPPDATA%\Programs\RBTray-rs\config.json` to prevent sensitive system processes (e.g., Task Manager) from being hidden.
6. **Dynamic Title Sync & Orphan Cleanup:**
   - Real-time periodic monitoring: updates tray tooltip when window title changes (e.g., download progress or chat notifications) and cleans up ghost icons if an app closes in the background.
7. **Completely Silent & Lightweight:**
   - Runs silently in the background with zero console windows (`windows_subsystem = "windows"`), consuming less than 2 MB of RAM.

---

## 📦 Installation & Setup

### Option 1: Official Windows Setup Wizard (.exe)
Download the standalone installer from the GitHub Releases page:
- **`dist/RBTray-rs-Setup-v0.2.0.exe`**

Features of the installer:
- Step-by-step setup wizard in English and Spanish.
- Option to automatically start RBTray-rs on Windows boot.
- Optional Desktop shortcut.
- Registered in Windows Settings (*Installed apps / Add or remove programs*) for clean one-click uninstall.

### Option 2: Quick PowerShell Install
Run in PowerShell:
```powershell
.\instalar.ps1
```

### Uninstallation
Run in PowerShell:
```powershell
.\desinstalar.ps1
```

---

## 📁 Project Structure

```text
rbtray-rs/
├── Cargo.toml               # Workspace configuration and release optimizations
├── build.ps1                # Automated build and packaging script
├── instalar.ps1             # PowerShell installer
├── desinstalar.ps1          # Clean uninstaller
├── installer.iss            # Inno Setup script for Setup .exe creation
├── assets/                  # App icon and GNU GPL v2 license
├── crates/
│   ├── rbhook/              # Global Windows hook DLL (cdylib)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs       # WH_MOUSE (with CSD/Electron detection) & WH_CALLWNDPROCRET
│   └── rbtray/              # Main background process (bin, windows_subsystem)
│       ├── Cargo.toml
│       └── src/main.rs      # Event loop, global hotkeys, dark mode, and tray logic
└── dist/                    # Compiled release binaries and Setup .exe
```

---

## 📖 How to Use

- **Using Mouse:** Right-click the minimize button (`_`) of any window (standard Win32 or modern Electron/CSD app).
- **Using Keyboard:**
  - Press **`Ctrl + Win + Down`** to minimize the active window to the tray.
  - Press **`Ctrl + Win + Up`** to restore the last minimized window.
  - Press **`Ctrl + Win + 0`** to restore all minimized windows.
- **Tray Menu:** Right-click any icon in the notification area for options (*Restore*, *Close*, *Restore All*, *Exit*).
- **Exit:** Right-click any tray icon and select *Exit RBTray-rs*, or execute `rbtray.exe --exit`.
