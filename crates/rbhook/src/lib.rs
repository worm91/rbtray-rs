use std::sync::atomic::{AtomicIsize, Ordering};
use windows_sys::Win32::Foundation::{
    BOOL, FALSE, HMODULE, HWND, LPARAM, LRESULT, POINT, RECT, TRUE, WPARAM,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, FindWindowW, GetAncestor, GetWindowLongPtrW, GetWindowRect, PostMessageW,
    SendMessageTimeoutW, SetWindowsHookExW, UnhookWindowsHookEx, CWPRETSTRUCT, GA_ROOT, GWL_STYLE,
    HHOOK, HTMINBUTTON, MOUSEHOOKSTRUCT, SMTO_ABORTIFHUNG, SWP_SHOWWINDOW, WH_CALLWNDPROCRET,
    WH_MOUSE, WINDOWPOS, WM_NCDESTROY, WM_NCHITTEST, WM_NCRBUTTONDOWN, WM_NCRBUTTONUP,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_WINDOWPOSCHANGED, WS_CAPTION, WS_CHILD, WS_THICKFRAME,
};

pub const WINDOW_CLASS_NAME: &[u16] = &[
    'R' as u16, 'B' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 'H' as u16,
    'o' as u16, 'o' as u16, 'k' as u16, 0,
];

pub const WM_ADDTRAY: u32 = 0x0401;
pub const WM_REMTRAY: u32 = 0x0402;
pub const WM_REFRTRAY: u32 = 0x0403;

static H_MOUSE: AtomicIsize = AtomicIsize::new(0);
static H_WNDPROC: AtomicIsize = AtomicIsize::new(0);
static LAST_HIT_HWND: AtomicIsize = AtomicIsize::new(0);

/// Evalúa si el clic ocurrió sobre el botón de minimizar, tanto para aplicaciones
/// tradicionales Win32 (HTMINBUTTON) como para aplicaciones modernas con barra personalizada
/// (Electron, Chrome, Discord, Spotify, VS Code, etc.).
unsafe fn is_minimize_hit(hwnd: HWND, w_hit_test: u32, pt: POINT) -> bool {
    // 1. Detección estándar Win32
    if w_hit_test == HTMINBUTTON as u32 {
        return true;
    }

    let root = GetAncestor(hwnd, GA_ROOT);
    if root.is_null() {
        return false;
    }

    let style = GetWindowLongPtrW(root, GWL_STYLE) as u32;
    if (style & WS_CHILD) != 0 {
        return false;
    }

    // 2. Preguntar a la ventana mediante WM_NCHITTEST (muchas apps CSD devuelven HTMINBUTTON)
    let l_param = ((pt.x as u32 & 0xFFFF) | ((pt.y as u32 & 0xFFFF) << 16)) as usize;
    let mut hit_result: usize = 0;
    let timeout_ok = SendMessageTimeoutW(
        root,
        WM_NCHITTEST,
        0,
        l_param as isize,
        SMTO_ABORTIFHUNG,
        30,
        &mut hit_result,
    );
    if timeout_ok != 0 && hit_result == HTMINBUTTON as usize {
        return true;
    }

    // 3. Heurística para apps modernas sin bordes tradicionales (Electron / frameless)
    // En Windows 10/11 estándar, los botones de ventana están en el cuadrante superior derecho.
    if (style & (WS_THICKFRAME | WS_CAPTION)) != 0 {
        let mut rc: RECT = std::mem::zeroed();
        if GetWindowRect(root, &mut rc) != 0 {
            let width = rc.right - rc.left;
            let height = rc.bottom - rc.top;
            if width > 320 && height > 180 {
                let in_title_bar = pt.y >= rc.top && pt.y <= (rc.top + 40);
                // Minimizar se ubica típicamente a la izquierda de Maximizar y Cerrar (~90px a 145px del borde derecho)
                let in_min_button = pt.x >= (rc.right - 145) && pt.x <= (rc.right - 90);
                if in_title_bar && in_min_button {
                    return true;
                }
            }
        }
    }

    false
}

unsafe extern "system" fn mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let hook = H_MOUSE.load(Ordering::Relaxed) as HHOOK;
    if n_code >= 0 {
        let msg = w_param as u32;
        let is_down = msg == WM_NCRBUTTONDOWN || msg == WM_RBUTTONDOWN;
        let is_up = msg == WM_NCRBUTTONUP || msg == WM_RBUTTONUP;

        if is_down || is_up {
            let info = *(l_param as *const MOUSEHOOKSTRUCT);
            let hit = is_minimize_hit(info.hwnd, info.wHitTestCode, info.pt);

            if is_down && hit {
                LAST_HIT_HWND.store(info.hwnd as isize, Ordering::Relaxed);
                return 1; // Consume el evento para evitar menús emergentes no deseados
            } else if is_up && hit {
                let last = LAST_HIT_HWND.swap(0, Ordering::Relaxed) as HWND;
                let root = GetAncestor(info.hwnd, GA_ROOT);
                let target = if root.is_null() { info.hwnd } else { root };

                if last == info.hwnd || last == target {
                    let target_wnd = FindWindowW(WINDOW_CLASS_NAME.as_ptr(), WINDOW_CLASS_NAME.as_ptr());
                    if !target_wnd.is_null() {
                        PostMessageW(target_wnd, WM_ADDTRAY, 0, target as LPARAM);
                    }
                }
                return 1;
            } else if is_down || is_up {
                LAST_HIT_HWND.store(0, Ordering::Relaxed);
            }
        }
    }
    CallNextHookEx(hook, n_code, w_param, l_param)
}

unsafe extern "system" fn call_wnd_ret_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let hook = H_WNDPROC.load(Ordering::Relaxed) as HHOOK;
    if n_code >= 0 {
        let msg = *(l_param as *const CWPRETSTRUCT);
        if msg.message == WM_NCDESTROY
            || (msg.message == WM_WINDOWPOSCHANGED
                && msg.lParam != 0
                && ((*(msg.lParam as *const WINDOWPOS)).flags & SWP_SHOWWINDOW) != 0)
        {
            let target_wnd = FindWindowW(WINDOW_CLASS_NAME.as_ptr(), WINDOW_CLASS_NAME.as_ptr());
            if !target_wnd.is_null() {
                PostMessageW(target_wnd, WM_REFRTRAY, 0, msg.hwnd as LPARAM);
            }
        }
    }
    CallNextHookEx(hook, n_code, w_param, l_param)
}

#[no_mangle]
pub unsafe extern "system" fn RegisterHook(h_lib: HMODULE) -> BOOL {
    let mouse = SetWindowsHookExW(WH_MOUSE, Some(mouse_proc), h_lib, 0);
    let wnd = SetWindowsHookExW(WH_CALLWNDPROCRET, Some(call_wnd_ret_proc), h_lib, 0);

    if mouse.is_null() || wnd.is_null() {
        UnRegisterHook();
        return FALSE;
    }

    H_MOUSE.store(mouse as isize, Ordering::SeqCst);
    H_WNDPROC.store(wnd as isize, Ordering::SeqCst);
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn UnRegisterHook() {
    let mouse = H_MOUSE.swap(0, Ordering::SeqCst) as HHOOK;
    if !mouse.is_null() {
        UnhookWindowsHookEx(mouse);
    }
    let wnd = H_WNDPROC.swap(0, Ordering::SeqCst) as HHOOK;
    if !wnd.is_null() {
        UnhookWindowsHookEx(wnd);
    }
}
