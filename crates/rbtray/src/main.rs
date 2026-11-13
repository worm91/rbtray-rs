#![windows_subsystem = "windows"]

use std::collections::HashMap;
use std::env;
use std::ptr;
use std::sync::Mutex;
use windows_sys::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::HBRUSH;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETVERSION,
    NIN_SELECT, NOTIFYICONDATAW, NOTIFYICON_VERSION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    FindWindowW, GetAncestor, GetClassLongPtrW, GetCursorPos, GetMessageW, GetWindowLongPtrW,
    GetWindowTextW, IsWindow, IsWindowVisible, LoadIconW, MessageBoxW, PostMessageW,
    PostQuitMessage, RegisterClassExW, RegisterWindowMessageW, SendMessageW, SetForegroundWindow,
    ShowWindow, TrackPopupMenu, TranslateMessage, GCLP_HICON, GCLP_HICONSM, GA_ROOT, GWL_EXSTYLE,
    GWL_STYLE, HICON, HMENU, IDI_APPLICATION, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
    MF_SEPARATOR, MF_STRING, SW_HIDE, SW_SHOW, TPM_BOTTOMALIGN, TPM_LEFTBUTTON, TPM_RIGHTALIGN,
    TPM_RIGHTBUTTON, WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU, WM_DESTROY, WM_GETICON, WM_USER,
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
const IDM_EXIT: usize = 0x1003;
const IDM_ABOUT: usize = 0x1004;

type RegisterHookFn = unsafe extern "system" fn(HMODULE) -> i32;
type UnRegisterHookFn = unsafe extern "system" fn();

struct TrayManager {
    next_id: u32,
    items: HashMap<u32, isize>,
    hwnd_to_id: HashMap<isize, u32>,
    menu_target: isize,
    _hook_lib: HMODULE,
    unregister_fn: Option<UnRegisterHookFn>,
}

unsafe impl Send for TrayManager {}
unsafe impl Sync for TrayManager {}

static MANAGER: Mutex<Option<TrayManager>> = Mutex::new(None);
static mut WM_TASKBAR_CREATED: u32 = 0;

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

    if mgr.hwnd_to_id.contains_key(&hwnd_val) {
        return;
    }

    let id = mgr.next_id;
    mgr.next_id += 1;
    mgr.items.insert(id, hwnd_val);
    mgr.hwnd_to_id.insert(hwnd_val, id);

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
            remove_from_tray(main_hwnd, id);
        }
    }
}

unsafe fn close_window_from_tray(hwnd: HWND) {
    PostMessageW(hwnd, WM_CLOSE, 0, 0);
}

unsafe fn show_context_menu(main_hwnd: HWND) {
    let h_menu: HMENU = CreatePopupMenu();
    if h_menu.is_null() {
        return;
    }

    let about_str: Vec<u16> = "Acerca de RBTray-rs\0".encode_utf16().collect();
    let exit_str: Vec<u16> = "Salir de RBTray-rs\0".encode_utf16().collect();
    let close_str: Vec<u16> = "Cerrar ventana\0".encode_utf16().collect();
    let restore_str: Vec<u16> = "Restaurar ventana\0".encode_utf16().collect();

    AppendMenuW(h_menu, MF_STRING, IDM_RESTORE, restore_str.as_ptr());
    AppendMenuW(h_menu, MF_STRING, IDM_CLOSE, close_str.as_ptr());
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
                        remove_from_tray(hwnd, id);
                    }
                }
            }
            0
        }
        WM_TRAYCMD => {
            let id = w_param as u32;
            let event = l_param as u32;
            let target_opt = {
                let lock = MANAGER.lock().unwrap();
                lock.as_ref().and_then(|m| m.items.get(&id).copied())
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
                IDM_ABOUT => {
                    let title: Vec<u16> = "Acerca de RBTray-rs\0".encode_utf16().collect();
                    let desc: Vec<u16> = concat!(
                        "RBTray-rs v0.1.0\n\n",
                        "Minimiza cualquier ventana a la bandeja del sistema con clic derecho sobre el botón minimizar.\n\n",
                        "Créditos:\n",
                        "• Basado en el concepto y versión original de RBTray (1998-2010) creada por Nikolay Redko y J.D. Purcell.\n",
                        "• Reimplementación moderna, segura y de alto rendimiento en Rust.\n\n",
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
            let windows_to_restore: Vec<HWND> = {
                let mut lock = MANAGER.lock().unwrap();
                if let Some(mgr) = lock.as_mut() {
                    if let Some(unreg) = mgr.unregister_fn {
                        unreg();
                    }
                    mgr.items.values().map(|&w| w as HWND).collect()
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
                    for (&id, &target_val) in &mgr.items {
                        add_to_tray(hwnd, id, target_val as HWND);
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

        {
            let mut lock = MANAGER.lock().unwrap();
            *lock = Some(TrayManager {
                next_id: 1,
                items: HashMap::new(),
                hwnd_to_id: HashMap::new(),
                menu_target: 0,
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

        // Bucle de despacho de mensajes
        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
