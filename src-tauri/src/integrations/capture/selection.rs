//! A short-lived Win32 overlay over an already-acquired snapshot. Coordinates
//! are physical pixels under a per-monitor-aware worker thread.
use super::pixels::SelectionRect;
use super::{Image, CAPTURE_CANCEL_MESSAGE};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Once,
};
use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::HiDpi::{
    GetDpiForWindow, SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, ReleaseCapture, SetCapture, VK_CONTROL, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::*;

struct SelectionState<'a> {
    image: &'a Image,
    overlay: &'a AtomicUsize,
    dimmed: Vec<u8>,
    rect: SelectionRect,
    dragging: Option<(i32, i32)>,
    accepted: bool,
}

pub(super) fn select(
    image: &Image,
    left: i32,
    top: i32,
    cancelled: &AtomicBool,
    overlay: &AtomicUsize,
) -> Result<Option<SelectionRect>, String> {
    if cancelled.load(Ordering::SeqCst) {
        return Ok(None);
    }
    let mut dimmed = image.pixels.clone();
    for pixel in dimmed.chunks_exact_mut(4) {
        for channel in &mut pixel[..3] {
            *channel = (*channel as f32 * 0.42) as u8;
        }
    }
    let mut state = SelectionState {
        image,
        overlay,
        dimmed,
        rect: SelectionRect {
            left: image.width / 4,
            top: image.height / 4,
            width: (image.width / 2).max(1),
            height: (image.height / 2).max(1),
        },
        dragging: None,
        accepted: false,
    };
    unsafe {
        let previous_dpi = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        static REGISTER: Once = Once::new();
        REGISTER.call_once(|| {
            let class = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                lpszClassName: w!("ProjectNotchAreaSelection"),
                hCursor: LoadCursorW(None, IDC_CROSS).unwrap_or_default(),
                ..Default::default()
            };
            RegisterClassW(&class);
        });
        let created = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            w!("ProjectNotchAreaSelection"),
            w!("Select screenshot area — Enter to copy, Escape to cancel"),
            WS_POPUP | WS_CLIPCHILDREN,
            left,
            top,
            image.width as i32,
            image.height as i32,
            None,
            None,
            None,
            Some(&mut state as *mut _ as *const _),
        );
        let hwnd = match created {
            Ok(hwnd) => hwnd,
            Err(error) => {
                SetThreadDpiAwarenessContext(previous_dpi);
                return Err(format!("Could not open area selection: {error}"));
            }
        };
        overlay.store(hwnd.0 as usize, Ordering::SeqCst);
        let dpi = GetDpiForWindow(hwnd).max(96) as i32;
        let button_width = 76 * dpi / 96;
        let button_height = 32 * dpi / 96;
        let gap = 8 * dpi / 96;
        // Real native buttons also expose Copy/Cancel to accessibility tools.
        let controls_font = CreateFontW(
            -14 * dpi / 96,
            0,
            0,
            0,
            500,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            0,
            w!("Segoe UI"),
        );
        for (id, title, column) in [(1usize, w!("Copy"), 2), (2usize, w!("Cancel"), 1)] {
            if let Ok(button) = CreateWindowExW(
                Default::default(),
                w!("BUTTON"),
                title,
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                image.width as i32 - column * (button_width + gap),
                10 * dpi / 96,
                button_width,
                button_height,
                Some(hwnd),
                Some(HMENU(id as *mut _)),
                None,
                None,
            ) {
                SendMessageW(
                    button,
                    WM_SETFONT,
                    Some(WPARAM(controls_font.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
        }
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(hwnd));
        if cancelled.load(Ordering::SeqCst) {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
        let mut message = MSG::default();
        loop {
            let received = GetMessageW(&mut message, None, 0, 0).0;
            if received <= 0 {
                break;
            }
            if message.message == WM_KEYDOWN {
                if message.wParam.0 == 9 && IsDialogMessageW(hwnd, &message).as_bool() {
                    continue;
                }
                if message.wParam.0 == 13 && GetDlgCtrlID(message.hwnd) == 2 {
                    SendMessageW(hwnd, WM_COMMAND, Some(WPARAM(2)), Some(LPARAM(0)));
                    continue;
                }
                // Selection keys keep working after a native button receives
                // focus; Space still activates that button normally.
                if matches!(message.wParam.0, 13 | 27 | 37..=40) {
                    SendMessageW(hwnd, WM_KEYDOWN, Some(message.wParam), Some(message.lParam));
                    continue;
                }
            }
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        overlay.store(0, Ordering::SeqCst);
        if IsWindow(Some(hwnd)).as_bool() {
            let _ = DestroyWindow(hwnd);
        }
        let _ = DeleteObject(controls_font.into());
        SetThreadDpiAwarenessContext(previous_dpi);
    }
    Ok(if state.accepted && !cancelled.load(Ordering::SeqCst) {
        Some(state.rect)
    } else {
        None
    })
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = &*(lparam.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        return LRESULT(1);
    }
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SelectionState;
    if pointer.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let state = &mut *pointer;
    let point = || {
        (
            (lparam.0 as u16 as i16) as i32,
            ((lparam.0 >> 16) as u16 as i16) as i32,
        )
    };
    match message {
        WM_PAINT => {
            paint(hwnd, state);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_LBUTTONDOWN => {
            state.dragging = Some(point());
            state.rect =
                SelectionRect::from_points(point(), point(), state.image.width, state.image.height);
            SetCapture(hwnd);
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_MOUSEMOVE if state.dragging.is_some() => {
            state.rect = SelectionRect::from_points(
                state.dragging.unwrap_or_default(),
                point(),
                state.image.width,
                state.image.height,
            );
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_LBUTTONUP if state.dragging.is_some() => {
            state.rect = SelectionRect::from_points(
                state.dragging.take().unwrap_or_default(),
                point(),
                state.image.width,
                state.image.height,
            );
            let _ = ReleaseCapture();
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_KEYDOWN => {
            match wparam.0 as u32 {
                0x1b => {
                    let _ = DestroyWindow(hwnd);
                }
                0x0d if state.rect.width > 0 && state.rect.height > 0 => {
                    state.accepted = true;
                    let _ = DestroyWindow(hwnd);
                }
                key @ 0x25..=0x28 if state.rect.width > 0 && state.rect.height > 0 => {
                    let step = if GetKeyState(VK_CONTROL.0 as i32) < 0 {
                        10
                    } else {
                        1
                    };
                    let (dx, dy) = match key {
                        0x25 => (-step, 0),
                        0x26 => (0, -step),
                        0x27 => (step, 0),
                        _ => (0, step),
                    };
                    state.rect.adjust(
                        dx,
                        dy,
                        GetKeyState(VK_SHIFT.0 as i32) < 0,
                        state.image.width,
                        state.image.height,
                    );
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
                _ => {}
            }
            LRESULT(0)
        }
        CAPTURE_CANCEL_MESSAGE if wparam.0 == state.overlay as *const _ as usize => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_COMMAND => {
            match wparam.0 & 0xffff {
                1 if state.rect.width > 0 && state.rect.height > 0 => {
                    state.accepted = true;
                    let _ = DestroyWindow(hwnd);
                }
                2 => {
                    let _ = DestroyWindow(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_DISPLAYCHANGE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            state.overlay.store(0, Ordering::SeqCst);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

unsafe fn paint(hwnd: HWND, state: &SelectionState) {
    let mut painting = PAINTSTRUCT::default();
    let dc = BeginPaint(hwnd, &mut painting);
    let bitmap = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: state.image.width as i32,
            biHeight: -(state.image.height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let draw = |pixels: &[u8]| {
        StretchDIBits(
            dc,
            0,
            0,
            state.image.width as i32,
            state.image.height as i32,
            0,
            0,
            state.image.width as i32,
            state.image.height as i32,
            Some(pixels.as_ptr() as *const _),
            &bitmap,
            DIB_RGB_COLORS,
            SRCCOPY,
        )
    };
    draw(&state.dimmed);
    let saved = SaveDC(dc);
    let rectangle = RECT {
        left: state.rect.left as i32,
        top: state.rect.top as i32,
        right: (state.rect.left + state.rect.width) as i32,
        bottom: (state.rect.top + state.rect.height) as i32,
    };
    IntersectClipRect(
        dc,
        rectangle.left,
        rectangle.top,
        rectangle.right,
        rectangle.bottom,
    );
    draw(&state.image.pixels);
    let _ = RestoreDC(dc, saved);
    let blue = CreateSolidBrush(COLORREF(0x00eeac83));
    let _ = FrameRect(dc, &rectangle, blue);
    let _ = DeleteObject(blue.into());
    let dpi = GetDpiForWindow(hwnd).max(96) as i32;
    let bar = RECT {
        left: 0,
        top: 0,
        right: state.image.width as i32,
        bottom: 52 * dpi / 96,
    };
    let brush = CreateSolidBrush(COLORREF(0x00181615));
    FillRect(dc, &bar, brush);
    let _ = DeleteObject(brush.into());
    let font = CreateFontW(
        -16 * dpi / 96,
        0,
        0,
        0,
        500,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        0,
        w!("Segoe UI"),
    );
    let previous_font = SelectObject(dc, font.into());
    SetBkMode(dc, TRANSPARENT);
    SetTextColor(dc, COLORREF(0x00f1eeec));
    let text_clip = SaveDC(dc);
    IntersectClipRect(
        dc,
        0,
        0,
        state.image.width as i32 - 180 * dpi / 96,
        bar.bottom,
    );
    let caption: Vec<u16> = format!("Drag to select  |  Enter: copy  |  Esc: cancel  |  Arrows: move  |  Shift+Arrows: resize  |  {} × {}", state.rect.width, state.rect.height).encode_utf16().collect();
    let _ = TextOutW(dc, 16 * dpi / 96, 16 * dpi / 96, &caption);
    let _ = RestoreDC(dc, text_clip);
    SelectObject(dc, previous_font);
    let _ = DeleteObject(font.into());
    let _ = EndPaint(hwnd, &painting);
}
