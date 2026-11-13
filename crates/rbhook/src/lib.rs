use std::sync::atomic::{AtomicIsize, Ordering};
use windows_sys::Win32::Foundation::{BOOL, FALSE, HMODULE, HWND, LPARAM, LRESULT, TRUE, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, FindWindowW, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    CWPRETSTRUCT, HHOOK, HTMINBUTTON, MOUSEHOOKSTRUCT, SWP_SHOWWINDOW, WH_CALLWNDPROCRET,
    WH_MOUSE, WINDOWPOS, WM_NCDESTROY, WM_NCRBUTTONDOWN, WM_NCRBUTTONUP, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_WINDOWPOSCHANGED,
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

unsafe extern "system" fn mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let hook = H_MOUSE.load(Ordering::Relaxed) as HHOOK;
    if n_code >= 0 {
        let msg = w_param as u32;
        if msg == WM_NCRBUTTONDOWN || msg == WM_NCRBUTTONUP {
            let info = *(l_param as *const MOUSEHOOKSTRUCT);
            let is_min_button = info.wHitTestCode == HTMINBUTTON as u32;

            if msg == WM_NCRBUTTONDOWN && is_min_button {
                LAST_HIT_HWND.store(info.hwnd as isize, Ordering::Relaxed);
                return 1; // Consume el evento
            } else if msg == WM_NCRBUTTONUP && is_min_button {
                let last = LAST_HIT_HWND.swap(0, Ordering::Relaxed) as HWND;
                if info.hwnd == last {
                    let target_wnd = FindWindowW(WINDOW_CLASS_NAME.as_ptr(), WINDOW_CLASS_NAME.as_ptr());
                    if !target_wnd.is_null() {
                        PostMessageW(target_wnd, WM_ADDTRAY, 0, info.hwnd as LPARAM);
                    }
                }
                return 1; // Consume el evento
            } else {
                LAST_HIT_HWND.store(0, Ordering::Relaxed);
            }
        } else if msg == WM_RBUTTONDOWN || msg == WM_RBUTTONUP {
            LAST_HIT_HWND.store(0, Ordering::Relaxed);
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
