#![windows_subsystem = "windows"]

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::ptr;
use std::sync::Mutex;
use windows_sys::Win32::Foundation::{
    CloseHandle, HMODULE, HWND, LPARAM, LRESULT, POINT, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::HBRUSH;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_WIN, VK_DOWN, VK_UP,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NIM_SETVERSION, NIN_SELECT, NOTIFYICONDATAW, NOTIFYICON_VERSION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    FindWindowW, GetAncestor, GetClassLongPtrW, GetCursorPos, GetForegroundWindow, GetMessageW,
    GetWindowLongPtrW, GetWindowTextW, GetWindowThreadProcessId, IsWindow, IsWindowVisible,
    KillTimer, LoadIconW, MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassExW,
    RegisterWindowMessageW, SendMessageW, SetForegroundWindow, SetTimer, ShowWindow, TrackPopupMenu,
    TranslateMessage, GCLP_HICON, GCLP_HICONSM, GA_ROOT, GWL_EXSTYLE, GWL_STYLE, HICON, HMENU,
    IDI_APPLICATION, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MF_SEPARATOR, MF_STRING, SW_HIDE,
    SW_SHOW, TPM_BOTTOMALIGN, TPM_LEFTBUTTON, TPM_RIGHTALIGN, TPM_RIGHTBUTTON, WM_CLOSE,
    WM_COMMAND, WM_CONTEXTMENU, WM_DESTROY, WM_GETICON, WM_HOTKEY, WM_TIMER, WM_USER,
    WNDCLASSEXW, WS_CHILD, WS_EX_MDICHILD, WS_OVERLAPPED,
};

const COLOR_WINDOW_SYS: u32 = 5;

const WINDOW_CLASS_NAME: &[u16] = &[
    'R' as u16, 'B' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 'H' as u16,
    'o' as u16, 'o' as u16, 'k' as u16, 0,
];

const DLL_NAME: &[u16] = &[
    'r' as u16, 'b' as u16, 'h' as u16, 'o' as u16, 'o' as u16, 'k' as u16, '.' as u16,
    'd' as u16, 'l' as u16, 'l' as u16, 0,
];

const TASKBAR_CREATED_STR: &[u16] = &[
    'T' as u16, 'a' as u16, 's' as u16, 'k' as u16, 'b' as u16, 'a' as u16, 'r' as u16,
    'C' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'd' as u16, 0,
];

// Mensajes personalizados IPC
const WM_ADDTRAY: u32 = 0x0401;
const WM_REMTRAY: u32 = 0x0402;
const WM_REFRTRAY: u32 = 0x0403;
const WM_TRAYCMD: u32 = 0x0404;

// Comandos de menú
const IDM_RESTORE: usize = 0x1001;
const IDM_CLOSE: usize = 0x1002;
const IDM_RESTORE_ALL: usize = 0x1003;
const IDM_CLOSE_ALL: usize = 0x1004;
const IDM_ABOUT: usize = 0x1005;
const IDM_EXIT: usize = 0x1006;

// IDs de Hotkeys
const HOTKEY_MINIMIZE: i32 = 1;
const HOTKEY_RESTORE_LAST: i32 = 2;
const HOTKEY_RESTORE_ALL: i32 = 3;

// ID de Timer
const TIMER_ID: usize = 100;

type RegisterHookFn = unsafe extern "system" fn(HMODULE) -> i32;
type UnRegisterHookFn = unsafe extern "system" fn();
type SetPreferredAppModeFn = unsafe extern "system" fn(i32) -> i32;
type FlushMenuThemesFn = unsafe extern "system" fn();

struct TrayItemInfo {
    hwnd: isize,
    last_title: String,
}

struct TrayManager {
    next_id: u32,
    items: HashMap<u32, TrayItemInfo>,
    hwnd_to_id: HashMap<isize, u32>,
    history_stack: Vec<isize>,
    menu_target: isize,
    blacklist: Vec<String>,
    _hook_lib: HMODULE,
    unregister_fn: Option<UnRegisterHookFn>,
}

unsafe impl Send for TrayManager {}
unsafe impl Sync for TrayManager {}

static MANAGER: Mutex<Option<TrayManager>> = Mutex::new(None);
static mut WM_TASKBAR_CREATED: u32 = 0;

/// Activa el soporte nativo de Modo Oscuro en menús contextuales de Windows 10/11
unsafe fn enable_dark_mode_menus() {
    let uxtheme_str: Vec<u16> = "uxtheme.dll\0".encode_utf16().collect();
    let h_uxtheme = LoadLibraryW(uxtheme_str.as_ptr());
    if !h_uxtheme.is_null() {
        // Ordinal 135: SetPreferredAppMode
        let set_mode_ptr = GetProcAddress(h_uxtheme, 135 as usize as *const u8);
        // Ordinal 136: FlushMenuThemes
        let flush_ptr = GetProcAddress(h_uxtheme, 136 as usize as *const u8);

        if let Some(set_mode) = set_mode_ptr {
            let set_mode_fn: SetPreferredAppModeFn = std::mem::transmute(set_mode);
            // 1 = AllowDark (sigue la preferencia del sistema operativo del usuario)
            set_mode_fn(1);
        }
        if let Some(flush) = flush_ptr {
            let flush_fn: FlushMenuThemesFn = std::mem::transmute(flush);
            flush_fn();
        }
    }
}

/// Obtiene la ruta del archivo de configuración
fn get_config_path() -> PathBuf {
    if let Ok(local) = env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local).join("Programs").join("RBTray-rs").join("config.json");
        return p;
    }
    PathBuf::from("config.json")
}

/// Carga la lista de exclusión o genera una por defecto
fn load_blacklist() -> Vec<String> {
    let default_list = vec![
        "taskmgr.exe".to_string(),
        "dwm.exe".to_string(),
        "shellexperiencehost.exe".to_string(),
        "searchhost.exe".to_string(),
        "lockapp.exe".to_string(),
    ];

    let path = get_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        let mut list = Vec::new();
        // Parser JSON ligero sin dependencias pesadas
        for line in content.lines() {
            let trimmed = line.trim().trim_matches(|c| c == '"' || c == ',' || c == '[' || c == ']');
            if trimmed.ends_with(".exe") {
                list.push(trimmed.to_lowercase());
            }
        }
        if !list.is_empty() {
            return list;
        }
    }

    // Escribir archivo por defecto si no existe
    let default_json = r#"{
  "description": "Lista de procesos que no seran minimizados al System Tray",
  "excludedProcesses": [
    "taskmgr.exe",
    "dwm.exe",
    "shellexperiencehost.exe",
    "searchhost.exe",
    "lockapp.exe"
  ],
  "hotkeys": {
    "minimizeForeground": "Win + Alt + Down",
    "restoreLast": "Win + Alt + Up",
    "restoreAll": "Win + Alt + R"
  }
}"#;
    let _ = fs::write(&path, default_json);
    default_list
}

/// Verifica si un proceso está en la lista de exclusión
unsafe fn is_window_excluded(hwnd: HWND, blacklist: &[String]) -> bool {
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == 0 {
        return false;
    }

    let h_proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if h_proc.is_null() {
        return false;
    }

    let mut buf = [0u16; 512];
    let mut size = buf.len() as u32;
    let mut is_excluded = false;

    if QueryFullProcessImageNameW(h_proc, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut size) != 0 {
        let path = String::from_utf16_lossy(&buf[..size as usize]).to_lowercase();
        for excluded in blacklist {
            if path.ends_with(excluded) {
                is_excluded = true;
                break;
            }
        }
    }

    CloseHandle(h_proc);
    is_excluded
}

unsafe fn get_window_title_string(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 256);
    if len > 0 {
        String::from_utf16_lossy(&buf[..len as usize])
    } else {
        String::from("Ventana")
    }
}

unsafe fn get_window_icon(hwnd: HWND) -> HICON {
    let mut icon = SendMessageW(hwnd, WM_GETICON, 2, 0) as HICON; // ICON_SMALL2
    if icon.is_null() {
        icon = SendMessageW(hwnd, WM_GETICON, 0, 0) as HICON; // ICON_SMALL
    }
    if icon.is_null() {
        icon = SendMessageW(hwnd, WM_GETICON, 1, 0) as HICON; // ICON_BIG
    }
    if icon.is_null() {
        icon = GetClassLongPtrW(hwnd, GCLP_HICONSM) as HICON;
    }
    if icon.is_null() {
        icon = GetClassLongPtrW(hwnd, GCLP_HICON) as HICON;
    }
    if icon.is_null() {
        icon = LoadIconW(ptr::null_mut(), IDI_APPLICATION);
    }
    icon
}

unsafe fn add_to_tray(main_hwnd: HWND, id: u32, target_hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = main_hwnd;
    nid.uID = id;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYCMD;
    nid.hIcon = get_window_icon(target_hwnd);
    GetWindowTextW(target_hwnd, nid.szTip.as_mut_ptr(), 128);

    Shell_NotifyIconW(NIM_ADD, &nid);
    nid.Anonymous.uVersion = NOTIFYICON_VERSION;
    Shell_NotifyIconW(NIM_SETVERSION, &nid);
}

unsafe fn remove_from_tray(main_hwnd: HWND, id: u32) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = main_hwnd;
    nid.uID = id;
    Shell_NotifyIconW(NIM_DELETE, &nid);
}

unsafe fn minimize_window(main_hwnd: HWND, mut hwnd: HWND) {
    if (GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_MDICHILD) != 0 {
        return;
    }
    if (GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_CHILD) != 0 {
        hwnd = GetAncestor(hwnd, GA_ROOT);
    }

    let hwnd_val = hwnd as isize;
    let mut lock = MANAGER.lock().unwrap();
    let mgr = match lock.as_mut() {
        Some(m) => m,
        None => return,
    };

    // Verificar lista de exclusión (Blacklist)
    if is_window_excluded(hwnd, &mgr.blacklist) {
        return;
    }

    if mgr.hwnd_to_id.contains_key(&hwnd_val) {
        return;
    }

    let id = mgr.next_id;
    mgr.next_id += 1;
    let title = get_window_title_string(hwnd);
    mgr.items.insert(id, TrayItemInfo {
        hwnd: hwnd_val,
        last_title: title,
    });
    mgr.hwnd_to_id.insert(hwnd_val, id);
    mgr.history_stack.push(hwnd_val);

    add_to_tray(main_hwnd, id, hwnd);
    ShowWindow(hwnd, SW_HIDE);
}

unsafe fn restore_window(main_hwnd: HWND, hwnd: HWND) {
    ShowWindow(hwnd, SW_SHOW);
    SetForegroundWindow(hwnd);

    let hwnd_val = hwnd as isize;
    let mut lock = MANAGER.lock().unwrap();
    if let Some(mgr) = lock.as_mut() {
        if let Some(id) = mgr.hwnd_to_id.remove(&hwnd_val) {
            mgr.items.remove(&id);
            mgr.history_stack.retain(|&x| x != hwnd_val);
            remove_from_tray(main_hwnd, id);
        }
    }
}

unsafe fn restore_all_windows(main_hwnd: HWND) {
    let windows_to_restore: Vec<HWND> = {
        let mut lock = MANAGER.lock().unwrap();
        if let Some(mgr) = lock.as_mut() {
            let list: Vec<HWND> = mgr.items.values().map(|it| it.hwnd as HWND).collect();
            for &id in mgr.items.keys() {
                remove_from_tray(main_hwnd, id);
            }
            mgr.items.clear();
            mgr.hwnd_to_id.clear();
            mgr.history_stack.clear();
            list
        } else {
            Vec::new()
        }
    };

    for w in windows_to_restore {
        ShowWindow(w, SW_SHOW);
        SetForegroundWindow(w);
    }
}

unsafe fn close_window_from_tray(hwnd: HWND) {
    PostMessageW(hwnd, WM_CLOSE, 0, 0);
}

unsafe fn close_all_windows_from_tray(main_hwnd: HWND) {
    let windows_to_close: Vec<HWND> = {
        let mut lock = MANAGER.lock().unwrap();
        if let Some(mgr) = lock.as_mut() {
            let list: Vec<HWND> = mgr.items.values().map(|it| it.hwnd as HWND).collect();
            for &id in mgr.items.keys() {
                remove_from_tray(main_hwnd, id);
            }
            mgr.items.clear();
            mgr.hwnd_to_id.clear();
            mgr.history_stack.clear();
            list
        } else {
            Vec::new()
        }
    };

    for w in windows_to_close {
        close_window_from_tray(w);
    }
}

unsafe fn show_context_menu(main_hwnd: HWND) {
    let h_menu: HMENU = CreatePopupMenu();
    if h_menu.is_null() {
        return;
    }

    let restore_str: Vec<u16> = "Restaurar esta ventana\0".encode_utf16().collect();
    let close_str: Vec<u16> = "Cerrar esta ventana\0".encode_utf16().collect();
    let restore_all_str: Vec<u16> = "Restaurar todas las ventanas\0".encode_utf16().collect();
    let close_all_str: Vec<u16> = "Cerrar todas las ventanas\0".encode_utf16().collect();
    let about_str: Vec<u16> = "Acerca de RBTray-rs\0".encode_utf16().collect();
    let exit_str: Vec<u16> = "Salir de RBTray-rs\0".encode_utf16().collect();

    AppendMenuW(h_menu, MF_STRING, IDM_RESTORE, restore_str.as_ptr());
    AppendMenuW(h_menu, MF_STRING, IDM_CLOSE, close_str.as_ptr());
    AppendMenuW(h_menu, MF_SEPARATOR, 0, ptr::null());
    AppendMenuW(h_menu, MF_STRING, IDM_RESTORE_ALL, restore_all_str.as_ptr());
    AppendMenuW(h_menu, MF_STRING, IDM_CLOSE_ALL, close_all_str.as_ptr());
    AppendMenuW(h_menu, MF_SEPARATOR, 0, ptr::null());
    AppendMenuW(h_menu, MF_STRING, IDM_ABOUT, about_str.as_ptr());
    AppendMenuW(h_menu, MF_STRING, IDM_EXIT, exit_str.as_ptr());

    let mut pt = POINT { x: 0, y: 0 };
    GetCursorPos(&mut pt);
    SetForegroundWindow(main_hwnd);
    TrackPopupMenu(
        h_menu,
        TPM_LEFTBUTTON | TPM_RIGHTBUTTON | TPM_RIGHTALIGN | TPM_BOTTOMALIGN,
        pt.x,
        pt.y,
        0,
        main_hwnd,
        ptr::null(),
    );
    PostMessageW(main_hwnd, WM_USER, 0, 0);
    DestroyMenu(h_menu);
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        WM_ADDTRAY => {
            minimize_window(hwnd, l_param as HWND);
            0
        }
        WM_REMTRAY => {
            restore_window(hwnd, l_param as HWND);
            0
        }
        WM_REFRTRAY => {
            let target = l_param as HWND;
            if IsWindow(target) == 0 || IsWindowVisible(target) != 0 {
                let target_val = target as isize;
                let mut lock = MANAGER.lock().unwrap();
                if let Some(mgr) = lock.as_mut() {
                    if let Some(id) = mgr.hwnd_to_id.remove(&target_val) {
                        mgr.items.remove(&id);
                        mgr.history_stack.retain(|&x| x != target_val);
                        remove_from_tray(hwnd, id);
                    }
                }
            }
            0
        }
        WM_TIMER => {
            // Refresco dinámico de títulos y limpieza de huérfanos cada 1.5s
            let mut lock = MANAGER.lock().unwrap();
            if let Some(mgr) = lock.as_mut() {
                let mut dead_ids = Vec::new();
                for (&id, item) in mgr.items.iter_mut() {
                    let target_hwnd = item.hwnd as HWND;
                    if IsWindow(target_hwnd) == 0 {
                        dead_ids.push(id);
                    } else {
                        let current_title = get_window_title_string(target_hwnd);
                        if current_title != item.last_title {
                            item.last_title = current_title;
                            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                            nid.hWnd = hwnd;
                            nid.uID = id;
                            nid.uFlags = NIF_TIP;
                            GetWindowTextW(target_hwnd, nid.szTip.as_mut_ptr(), 128);
                            Shell_NotifyIconW(NIM_MODIFY, &nid);
                        }
                    }
                }

                for dead_id in dead_ids {
                    if let Some(item) = mgr.items.remove(&dead_id) {
                        mgr.hwnd_to_id.remove(&item.hwnd);
                        mgr.history_stack.retain(|&x| x != item.hwnd);
                        remove_from_tray(hwnd, dead_id);
                    }
                }
            }
            0
        }
        WM_HOTKEY => {
            let hotkey_id = w_param as i32;
            match hotkey_id {
                HOTKEY_MINIMIZE => {
                    let fg = GetForegroundWindow();
                    if !fg.is_null() && fg != hwnd {
                        minimize_window(hwnd, fg);
                    }
                }
                HOTKEY_RESTORE_LAST => {
                    let last_hwnd = {
                        let lock = MANAGER.lock().unwrap();
                        lock.as_ref().and_then(|m| m.history_stack.last().copied())
                    };
                    if let Some(last) = last_hwnd {
                        restore_window(hwnd, last as HWND);
                    }
                }
                HOTKEY_RESTORE_ALL => {
                    restore_all_windows(hwnd);
                }
                _ => {}
            }
            0
        }
        WM_TRAYCMD => {
            let id = w_param as u32;
            let event = l_param as u32;
            let target_opt = {
                let lock = MANAGER.lock().unwrap();
                lock.as_ref().and_then(|m| m.items.get(&id).map(|it| it.hwnd))
            };

            if let Some(target_val) = target_opt {
                let target = target_val as HWND;
                if event == NIN_SELECT {
                    restore_window(hwnd, target);
                } else if event == WM_CONTEXTMENU {
                    {
                        let mut lock = MANAGER.lock().unwrap();
                        if let Some(mgr) = lock.as_mut() {
                            mgr.menu_target = target_val;
                        }
                    }
                    show_context_menu(hwnd);
                }
            }
            0
        }
        WM_COMMAND => {
            let cmd = (w_param & 0xFFFF) as usize;
            let menu_target = {
                let lock = MANAGER.lock().unwrap();
                lock.as_ref().map(|m| m.menu_target).unwrap_or(0)
            };

            match cmd {
                IDM_RESTORE => {
                    if menu_target != 0 {
                        restore_window(hwnd, menu_target as HWND);
                    }
                }
                IDM_CLOSE => {
                    if menu_target != 0 {
                        close_window_from_tray(menu_target as HWND);
                    }
                }
                IDM_RESTORE_ALL => {
                    restore_all_windows(hwnd);
                }
                IDM_CLOSE_ALL => {
                    close_all_windows_from_tray(hwnd);
                }
                IDM_ABOUT => {
                    let title: Vec<u16> = "Acerca de RBTray-rs\0".encode_utf16().collect();
                    let desc: Vec<u16> = concat!(
                        "RBTray-rs v0.2.0\n\n",
                        "Minimiza cualquier ventana al área de notificación de Windows con clic derecho o atajos de teclado.\n\n",
                        "Desarrollado y modernizado por:\n",
                        "• Luis Alberto Garcia (@worm91) — Reimplementación en Rust, soporte CSD/Electron, atajos y UX moderna.\n\n",
                        "Basado en la idea original de:\n",
                        "• Nikolay Redko y J.D. Purcell (RBTray clásico en C++, 1998–2010).\n\n",
                        "Licencia: GNU GPL v2"
                    )
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                    MessageBoxW(hwnd, desc.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
                }
                IDM_EXIT => {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                }
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            KillTimer(hwnd, TIMER_ID);
            UnregisterHotKey(hwnd, HOTKEY_MINIMIZE);
            UnregisterHotKey(hwnd, HOTKEY_RESTORE_LAST);
            UnregisterHotKey(hwnd, HOTKEY_RESTORE_ALL);

            let windows_to_restore: Vec<HWND> = {
                let mut lock = MANAGER.lock().unwrap();
                if let Some(mgr) = lock.as_mut() {
                    if let Some(unreg) = mgr.unregister_fn {
                        unreg();
                    }
                    mgr.items.values().map(|it| it.hwnd as HWND).collect()
                } else {
                    Vec::new()
                }
            };

            for w in windows_to_restore {
                ShowWindow(w, SW_SHOW);
            }

            PostQuitMessage(0);
            0
        }
        _ => {
            if WM_TASKBAR_CREATED != 0 && msg == WM_TASKBAR_CREATED {
                let lock = MANAGER.lock().unwrap();
                if let Some(mgr) = lock.as_ref() {
                    for (&id, item) in &mgr.items {
                        add_to_tray(hwnd, id, item.hwnd as HWND);
                    }
                }
                0
            } else {
                DefWindowProcW(hwnd, msg, w_param, l_param)
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    unsafe {
        let existing = FindWindowW(WINDOW_CLASS_NAME.as_ptr(), WINDOW_CLASS_NAME.as_ptr());
        if !existing.is_null() {
            if args.iter().any(|a| a == "--exit") {
                PostMessageW(existing, WM_CLOSE, 0, 0);
            } else {
                let msg: Vec<u16> = "RBTray-rs ya se encuentra en ejecución.\0".encode_utf16().collect();
                let title: Vec<u16> = "RBTray-rs\0".encode_utf16().collect();
                MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
            }
            return;
        }

        // Habilitar Modo Oscuro nativo en menús
        enable_dark_mode_menus();

        // Cargar DLL del hook
        let h_lib = LoadLibraryW(DLL_NAME.as_ptr());
        if h_lib.is_null() {
            let err_msg: Vec<u16> = "No se pudo cargar rbhook.dll. Asegúrate de que esté en el mismo directorio.\0"
                .encode_utf16()
                .collect();
            let title: Vec<u16> = "Error\0".encode_utf16().collect();
            MessageBoxW(ptr::null_mut(), err_msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
            return;
        }

        let reg_fn_ptr = GetProcAddress(h_lib, b"RegisterHook\0".as_ptr());
        let unreg_fn_ptr = GetProcAddress(h_lib, b"UnRegisterHook\0".as_ptr());

        if reg_fn_ptr.is_none() || unreg_fn_ptr.is_none() {
            let err_msg: Vec<u16> = "Exportaciones inválidas en rbhook.dll.\0".encode_utf16().collect();
            let title: Vec<u16> = "Error\0".encode_utf16().collect();
            MessageBoxW(ptr::null_mut(), err_msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
            return;
        }

        let register_hook: RegisterHookFn = std::mem::transmute(reg_fn_ptr);
        let unregister_hook: UnRegisterHookFn = std::mem::transmute(unreg_fn_ptr);

        if register_hook(h_lib) == 0 {
            let err_msg: Vec<u16> = "Error al inicializar el hook global de Windows.\0".encode_utf16().collect();
            let title: Vec<u16> = "Error\0".encode_utf16().collect();
            MessageBoxW(ptr::null_mut(), err_msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
            return;
        }

        let blacklist = load_blacklist();

        {
            let mut lock = MANAGER.lock().unwrap();
            *lock = Some(TrayManager {
                next_id: 1,
                items: HashMap::new(),
                hwnd_to_id: HashMap::new(),
                history_stack: Vec::new(),
                menu_target: 0,
                blacklist,
                _hook_lib: h_lib,
                unregister_fn: Some(unregister_hook),
            });
        }

        let h_instance = GetModuleHandleW(ptr::null());

        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(wnd_proc);
        wc.hInstance = h_instance;
        wc.hbrBackground = (COLOR_WINDOW_SYS + 1) as HBRUSH;
        wc.lpszClassName = WINDOW_CLASS_NAME.as_ptr();

        if RegisterClassExW(&wc) == 0 {
            let err_msg: Vec<u16> = "Error registrando clase de ventana.\0".encode_utf16().collect();
            let title: Vec<u16> = "Error\0".encode_utf16().collect();
            MessageBoxW(ptr::null_mut(), err_msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
            return;
        }

        let main_hwnd = CreateWindowExW(
            0,
            WINDOW_CLASS_NAME.as_ptr(),
            WINDOW_CLASS_NAME.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            h_instance,
            ptr::null(),
        );

        if main_hwnd.is_null() {
            return;
        }

        WM_TASKBAR_CREATED = RegisterWindowMessageW(TASKBAR_CREATED_STR.as_ptr());

        // Registrar Atajos Globales de Teclado
        // Win + Alt + Down: Minimizar ventana activa
        RegisterHotKey(main_hwnd, HOTKEY_MINIMIZE, (MOD_ALT | MOD_WIN) as u32, VK_DOWN as u32);
        // Win + Alt + Up: Restaurar última ventana minimizada
        RegisterHotKey(main_hwnd, HOTKEY_RESTORE_LAST, (MOD_ALT | MOD_WIN) as u32, VK_UP as u32);
        // Win + Alt + R: Restaurar todas las ventanas
        RegisterHotKey(main_hwnd, HOTKEY_RESTORE_ALL, (MOD_ALT | MOD_WIN) as u32, 0x52); // 'R'

        // Iniciar temporizador de sincronización dinámica cada 1500ms
        SetTimer(main_hwnd, TIMER_ID, 1500, None);

        // Bucle de despacho de mensajes
        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
