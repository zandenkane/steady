/// Windows backend using WH_MOUSE_LL low-level mouse hook.
///
/// Installs a system-wide hook that intercepts mouse move events,
/// passes them through the filter pipeline, suppresses the original
/// event, and reinjects the filtered position via SendInput.
///
/// The reinjection guard checks both LLMHF_INJECTED and a magic
/// dwExtraInfo value to prevent infinite recursion.
use crate::backend::types::{BackendAction, RawPointerEvent};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_MOUSE, MOUSEINPUT, MOUSE_EVENT_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, GetSystemMetrics, SetWindowsHookExW,
    UnhookWindowsHookEx, MSG, MSLLHOOKSTRUCT, SM_CXSCREEN, SM_CYSCREEN, WH_MOUSE_LL, WM_MOUSEMOVE,
};

/// Magic value written to dwExtraInfo on injected events.
const MAGIC_EXTRA_INFO: usize = 0x5445_4144_5949;

/// Flag bit for injected events in MSLLHOOKSTRUCT.flags.
const LLMHF_INJECTED_BIT: u32 = 0x01;

// MOUSE_EVENT_FLAGS constants as raw u32 for bitwise operations.
const MEF_MOVE: u32 = 0x0001;
const MEF_ABSOLUTE: u32 = 0x8000;
const MEF_LEFTDOWN: u32 = 0x0002;
const MEF_LEFTUP: u32 = 0x0004;

thread_local! {
    static HOOK_STATE: RefCell<Option<HookState>> = const { RefCell::new(None) };
}

struct HookState {
    start_time: Instant,
    callback: Box<dyn FnMut(RawPointerEvent) -> Vec<BackendAction>>,
}

pub struct WindowsBackend {
    pub stop_flag: Arc<AtomicBool>,
}

impl Default for WindowsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowsBackend {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the hook loop. Blocks until shutdown is requested.
    pub fn run<F>(&mut self, callback: F) -> Result<(), String>
    where
        F: FnMut(RawPointerEvent) -> Vec<BackendAction> + 'static,
    {
        let stop = self.stop_flag.clone();

        HOOK_STATE.with(|state| {
            *state.borrow_mut() = Some(HookState {
                start_time: Instant::now(),
                callback: Box::new(callback),
            });
        });

        let hook = unsafe {
            SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0)
                .map_err(|e| format!("SetWindowsHookExW failed: {}", e))?
        };

        eprintln!("steady: hook installed, filtering active");
        eprintln!("steady: press Ctrl+C to stop");

        let mut msg = MSG::default();
        while !stop.load(Ordering::Relaxed) {
            let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
            if ret.0 <= 0 {
                break;
            }
            unsafe {
                let _ = DispatchMessageW(&msg);
            }
        }

        unsafe {
            let _ = UnhookWindowsHookEx(hook);
        }

        HOOK_STATE.with(|state| {
            *state.borrow_mut() = None;
        });

        eprintln!("steady: hook removed, shutting down");
        Ok(())
    }

    pub fn request_stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    fn build_input(dx: i32, dy: i32, flags_raw: u32) -> INPUT {
        let mut input: INPUT = unsafe { std::mem::zeroed() };
        input.r#type = INPUT_MOUSE;
        input.Anonymous.mi = MOUSEINPUT {
            dx,
            dy,
            mouseData: 0,
            dwFlags: MOUSE_EVENT_FLAGS(flags_raw),
            time: 0,
            dwExtraInfo: MAGIC_EXTRA_INFO,
        };
        input
    }

    fn to_absolute(x: i32, y: i32) -> Option<(i32, i32)> {
        let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        if sw == 0 || sh == 0 {
            return None;
        }
        let ax = ((x as i64 * 65535) / sw as i64) as i32;
        let ay = ((y as i64 * 65535) / sh as i64) as i32;
        Some((ax, ay))
    }

    fn send_mouse_move(x: i32, y: i32) {
        if let Some((ax, ay)) = Self::to_absolute(x, y) {
            let input = Self::build_input(ax, ay, MEF_MOVE | MEF_ABSOLUTE);
            unsafe {
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    fn send_click(x: i32, y: i32) {
        Self::send_mouse_move(x, y);
        if let Some((ax, ay)) = Self::to_absolute(x, y) {
            let down = Self::build_input(ax, ay, MEF_LEFTDOWN | MEF_ABSOLUTE);
            let up = Self::build_input(ax, ay, MEF_LEFTUP | MEF_ABSOLUTE);
            unsafe {
                SendInput(&[down, up], std::mem::size_of::<INPUT>() as i32);
            }
        }
    }
}

unsafe extern "system" fn mouse_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code < 0 {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    let msg_id = w_param.0 as u32;
    if msg_id != WM_MOUSEMOVE {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    let info = &*(l_param.0 as *const MSLLHOOKSTRUCT);

    // Guard: skip events we injected ourselves.
    if (info.flags & LLMHF_INJECTED_BIT) != 0 || info.dwExtraInfo == MAGIC_EXTRA_INFO {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    let suppress = HOOK_STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(ref mut hs) = *state {
            let elapsed = hs.start_time.elapsed().as_secs_f64();
            let raw = RawPointerEvent {
                x: info.pt.x as f64,
                y: info.pt.y as f64,
                timestamp: elapsed,
                injected: false,
            };

            let actions = (hs.callback)(raw);
            let mut did_move = false;

            for action in actions {
                match action {
                    BackendAction::MoveTo(x, y) => {
                        WindowsBackend::send_mouse_move(x as i32, y as i32);
                        did_move = true;
                    }
                    BackendAction::Click(x, y) => {
                        WindowsBackend::send_click(x as i32, y as i32);
                    }
                }
            }

            did_move
        } else {
            false
        }
    });

    if suppress {
        LRESULT(1)
    } else {
        CallNextHookEx(None, n_code, w_param, l_param)
    }
}
