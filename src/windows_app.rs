#![allow(unsafe_op_in_unsafe_fn)]

use forza_key_overlay::overlay_state::{DisplayKey, KeySnapshot, OverlayState};
use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, CreateSolidBrush, DeleteObject, Ellipse, EndPaint, FillRect,
    GetStockObject, HBRUSH, HDC, HGDIOBJ, HOLLOW_BRUSH, InvalidateRect, LineTo, MoveToEx,
    PAINTSTRUCT, PS_SOLID, Rectangle, SelectObject, SetBkMode, SetTextColor, TRANSPARENT, TextOutW,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_SHIFT, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW,
    DispatchMessageW, GWL_EXSTYLE, GWLP_USERDATA, GetMessageW, GetWindowLongPtrW, HMENU, HTCAPTION,
    HWND_TOPMOST, IDC_ARROW, KillTimer, LWA_COLORKEY, LoadCursorW, MSG, PostQuitMessage,
    RegisterClassExW, SW_HIDE, SW_SHOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SetLayeredWindowAttributes, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    TranslateMessage, WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY, WM_NCCREATE, WM_NCHITTEST, WM_PAINT,
    WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::{PCWSTR, Result};

const TIMER_ID: usize = 1;
const TIMER_MS: u32 = 16;
const TRANSPARENT_COLOR: COLORREF = COLORREF(0);
const WINDOW_W: i32 = 260;
const WINDOW_H: i32 = 520;
const KEY: i32 = 54;
const GAP: i32 = 8;
const SMALL_W: i32 = 84;
const SPACE_W: i32 = KEY * 3 + GAP * 2;
const LEFT: i32 = 24;
const TOP: i32 = 220;

struct AppState {
    overlay: OverlayState,
    accepts_mouse: bool,
}

pub fn run() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide("ForzaKeyOverlayWindow");

        let wnd_class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: HINSTANCE(instance.0),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(GetStockObject(HOLLOW_BRUSH).0),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wnd_class);

        let mut app_state = Box::new(AppState {
            overlay: OverlayState::new(),
            accepts_mouse: false,
        });
        let state_ptr = app_state.as_mut() as *mut AppState;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(
                WS_EX_LAYERED.0
                    | WS_EX_TRANSPARENT.0
                    | WS_EX_TOPMOST.0
                    | WS_EX_TOOLWINDOW.0
                    | WS_EX_NOACTIVATE.0,
            ),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide("Forza Key Overlay").as_ptr()),
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WINDOW_W,
            WINDOW_H,
            None,
            HMENU::default(),
            HINSTANCE(instance.0),
            Some(state_ptr.cast::<c_void>()),
        )?;

        Box::leak(app_state);

        SetLayeredWindowAttributes(hwnd, TRANSPARENT_COLOR, 255, LWA_COLORKEY)?;
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )?;
        let _ = ShowWindow(hwnd, SW_SHOW);
        SetTimer(hwnd, TIMER_ID, TIMER_MS, None);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    Ok(())
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_NCCREATE => {
                let create = lparam.0 as *const CREATESTRUCTW;
                let state = (*create).lpCreateParams as *mut AppState;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
                LRESULT(1)
            }
            WM_CREATE => LRESULT(0),
            WM_TIMER => {
                if wparam.0 == TIMER_ID {
                    if let Some(state) = app_state(hwnd) {
                        poll_keys(state);
                        sync_move_mode(hwnd, state);
                        let _ = ShowWindow(
                            hwnd,
                            if state.overlay.visible() {
                                SW_SHOW
                            } else {
                                SW_HIDE
                            },
                        );
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
                LRESULT(0)
            }
            WM_PAINT => {
                paint(hwnd);
                LRESULT(0)
            }
            WM_NCHITTEST => {
                if let Some(state) = app_state(hwnd) {
                    if state.overlay.move_mode() {
                        return LRESULT(HTCAPTION as isize);
                    }
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_DESTROY => {
                let _ = KillTimer(hwnd, TIMER_ID);
                let ptr = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *mut AppState;
                if !ptr.is_null() {
                    drop(Box::from_raw(ptr));
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn app_state(hwnd: HWND) -> Option<&'static mut AppState> {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
    ptr.as_mut()
}

unsafe fn poll_keys(state: &mut AppState) {
    state.overlay.update_toggle_key(key_down(u16::from(b'U')));
    state.overlay.update_move_key(key_down(u16::from(b'M')));

    let mut keys = Vec::new();
    for (display_key, vk) in [
        (DisplayKey::W, u16::from(b'W')),
        (DisplayKey::A, u16::from(b'A')),
        (DisplayKey::S, u16::from(b'S')),
        (DisplayKey::D, u16::from(b'D')),
        (DisplayKey::Space, VK_SPACE.0),
        (DisplayKey::Shift, VK_SHIFT.0),
        (DisplayKey::Ctrl, VK_CONTROL.0),
    ] {
        if key_down(vk) {
            keys.push(display_key);
        }
    }

    state.overlay.update_keys(KeySnapshot::from_iter(keys));
}

unsafe fn sync_move_mode(hwnd: HWND, state: &mut AppState) {
    let should_accept_mouse = state.overlay.move_mode();
    if state.accepts_mouse == should_accept_mouse {
        return;
    }

    let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    let transparent = WS_EX_TRANSPARENT.0 as isize;
    let next = if should_accept_mouse {
        current & !transparent
    } else {
        current | transparent
    };

    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next);
    let _ = SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
    );
    state.accepts_mouse = should_accept_mouse;
}

unsafe fn key_down(vk: u16) -> bool {
    (GetAsyncKeyState(i32::from(vk)) as u16 & 0x8000) != 0
}

unsafe fn paint(hwnd: HWND) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    let bg = CreateSolidBrush(TRANSPARENT_COLOR);
    FillRect(
        hdc,
        &RECT {
            left: 0,
            top: 0,
            right: WINDOW_W,
            bottom: WINDOW_H,
        },
        bg,
    );
    let _ = DeleteObject(HGDIOBJ(bg.0));

    if let Some(state) = app_state(hwnd) {
        draw_key(
            hdc,
            DisplayKey::W,
            "W",
            LEFT + KEY + GAP,
            TOP,
            KEY,
            KEY,
            state,
        );
        draw_key(
            hdc,
            DisplayKey::A,
            "A",
            LEFT,
            TOP + KEY + GAP,
            KEY,
            KEY,
            state,
        );
        draw_key(
            hdc,
            DisplayKey::S,
            "S",
            LEFT + KEY + GAP,
            TOP + KEY + GAP,
            KEY,
            KEY,
            state,
        );
        draw_key(
            hdc,
            DisplayKey::D,
            "D",
            LEFT + (KEY + GAP) * 2,
            TOP + KEY + GAP,
            KEY,
            KEY,
            state,
        );
        draw_key(
            hdc,
            DisplayKey::Space,
            "SPACE",
            LEFT,
            TOP + (KEY + GAP) * 2,
            SPACE_W,
            44,
            state,
        );
        draw_key(
            hdc,
            DisplayKey::Shift,
            "SHIFT",
            LEFT,
            TOP + (KEY + GAP) * 2 + 44 + GAP,
            SMALL_W,
            42,
            state,
        );
        draw_key(
            hdc,
            DisplayKey::Ctrl,
            "CTRL",
            LEFT + SMALL_W + GAP,
            TOP + (KEY + GAP) * 2 + 44 + GAP,
            SMALL_W,
            42,
            state,
        );
    }

    let _ = EndPaint(hwnd, &ps);
}

unsafe fn draw_key(
    hdc: HDC,
    key: DisplayKey,
    label: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    state: &AppState,
) {
    let pressed = state.overlay.is_pressed(key);
    let border = if pressed {
        rgb(231, 251, 255)
    } else {
        rgb(168, 219, 240)
    };
    let text = rgb(240, 251, 255);
    let pen_width = if pressed { 4 } else { 2 };

    if pressed {
        draw_glow(hdc, x, y, w, h);
    }

    let pen = CreatePen(PS_SOLID, pen_width, border);
    let old_brush = SelectObject(hdc, GetStockObject(HOLLOW_BRUSH));
    let old_pen = SelectObject(hdc, HGDIOBJ(pen.0));

    let _ = Rectangle(hdc, x, y, x + w, y + h);

    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
    let _ = DeleteObject(HGDIOBJ(pen.0));

    let highlight_color = if pressed {
        rgb(255, 255, 255)
    } else {
        rgb(245, 253, 255)
    };
    let highlight = CreatePen(PS_SOLID, 1, highlight_color);
    let old_pen = SelectObject(hdc, HGDIOBJ(highlight.0));
    let _ = MoveToEx(hdc, x + 6, y + 5, Some(&mut POINT::default()));
    let _ = LineTo(hdc, x + w - 6, y + 5);
    SelectObject(hdc, old_pen);
    let _ = DeleteObject(HGDIOBJ(highlight.0));

    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, text);

    let label_w = label.len() as i32 * 8;
    let label_h = 16;
    let wide_label = wide(label);
    let _ = TextOutW(
        hdc,
        x + (w - label_w) / 2,
        y + (h - label_h) / 2,
        &wide_label[..wide_label.len() - 1],
    );
}

unsafe fn draw_glow(hdc: HDC, x: i32, y: i32, w: i32, h: i32) {
    let glow = CreatePen(PS_SOLID, 1, rgb(108, 220, 255));
    let old_pen = SelectObject(hdc, HGDIOBJ(glow.0));
    let old_brush = SelectObject(hdc, GetStockObject(HOLLOW_BRUSH));

    for offset in 2..=7 {
        let _ = Rectangle(hdc, x - offset, y - offset, x + w + offset, y + h + offset);
    }
    let _ = Ellipse(hdc, x - 1, y - 1, x, y);

    SelectObject(hdc, old_brush);
    SelectObject(hdc, old_pen);
    let _ = DeleteObject(HGDIOBJ(glow.0));
}

fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(u32::from(r) | (u32::from(g) << 8) | (u32::from(b) << 16))
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
