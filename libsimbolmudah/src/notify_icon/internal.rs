/// Taken from https://github.com/tauri-apps/tray-icon/blob/d4078696edba67b0ab42cef67e6a421a0332c96f/src/platform_impl/windows/mod.rs
/// with modifications.
use std::cell::RefCell;

use windows::{
    core::{h, w, Result, Weak, HSTRING, PCWSTR},
    Foundation::EventRegistrationToken,
    Graphics::PointInt32,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::SystemServices::IMAGE_DOS_HEADER,
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD,
                NIM_DELETE, NIM_SETVERSION, NOTIFYICONDATAW, NOTIFYICONDATAW_0,
                NOTIFYICON_VERSION_4,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, LoadIconW, RegisterClassW,
                CW_USEDEFAULT, HICON, HMENU, HWND_MESSAGE, IDI_WARNING, WINDOW_EX_STYLE, WNDCLASSW,
                WS_MINIMIZEBOX, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
            },
        },
    },
};

use crate::{bindings, delegate_storage::DelegateStorage, fail_message, get_strong_ref};

use super::{counter::GLOBAL_COUNTER, menu::NotifyIconMenu};

const WM_USER_TRAYICON: u32 = 0x1772;
// const WM_USER_UPDATE_TRAYMENU: u32 = 6003;
// const WM_USER_UPDATE_TRAYICON: u32 = 6004;
// const WM_USER_SHOW_TRAYICON: u32 = 6005;
// const WM_USER_HIDE_TRAYICON: u32 = 6006;
// const WM_USER_UPDATE_TRAYTOOLTIP: u32 = 6007;
// const WM_USER_LEAVE_TIMER_ID: u32 = 6008;

thread_local! {
    pub(super) static INTERNAL_NOTIFYICON: RefCell<Option<NotifyIconInternal>> = const { RefCell::new(None) };
}

pub(super) struct NotifyIconInternal {
    h_wnd: HWND,
    internal_id: u32,
    h_menu: Option<NotifyIconMenu>,
    pub(super) report_selected: DelegateStorage<bindings::NotifyIcon, PointInt32>,
    pub(super) on_state_changed_token: EventRegistrationToken,
    pub(super) parent: Weak<bindings::NotifyIcon>,
}

impl NotifyIconInternal {
    pub(super) fn create_for_thread(parent: Weak<bindings::NotifyIcon>) -> Result<()> {
        let internal_id = GLOBAL_COUNTER.next();

        let class_name = w!("LibSimbolMudah.NotifyIcon");
        let h_instance = get_instance_handle();
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(notify_proc),
            lpszClassName: class_name,
            hInstance: h_instance,
            ..Default::default()
        };

        if unsafe { RegisterClassW(&wnd_class) } == 0 {
            return Err(fail_message("Failed to register window class"));
        }

        let h_wnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                PCWSTR::null(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_MINIMIZEBOX,
                CW_USEDEFAULT,
                0,
                CW_USEDEFAULT,
                0,
                HWND_MESSAGE,
                HMENU::default(),
                h_instance,
                None,
            )
        }?;

        let h_menu = NotifyIconMenu::new(false)?;

        let res = Self {
            h_wnd,
            internal_id,
            h_menu: Some(h_menu),
            report_selected: DelegateStorage::new(),
            on_state_changed_token: EventRegistrationToken::default(),
            parent,
        };

        let h_icon = unsafe { LoadIconW(None, IDI_WARNING) }?;
        res.register_notify_icon(h_icon, h!("simbolmudah (not listening)"))?;

        INTERNAL_NOTIFYICON.set(Some(res));
        Ok(())
    }

    fn register_notify_icon(&self, h_icon: HICON, tooltip: &HSTRING) -> Result<()> {
        let mut sz_tip = [0; 128];
        let tip = tooltip.as_wide();
        sz_tip[..tip.len().min(128)].copy_from_slice(&tip[..tip.len().min(128)]);

        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.h_wnd,
            uID: self.internal_id,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: WM_USER_TRAYICON,
            hIcon: h_icon,
            szTip: sz_tip,
            Anonymous: NOTIFYICONDATAW_0 {
                uVersion: NOTIFYICON_VERSION_4,
            },
            ..Default::default()
        };

        if unsafe { Shell_NotifyIconW(NIM_ADD, &nid) }.into() {
            if unsafe { Shell_NotifyIconW(NIM_SETVERSION, &nid) }.into() {
                Ok(())
            } else {
                let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) };
                Err(fail_message(
                    "Failed to set notify icon version, deleting icon (Shell_NotifyIconW)",
                ))
            }
        } else {
            Err(fail_message(
                "Failed to register notify icon (Shell_NotifyIconW)",
            ))
        }
    }

    fn remove_tray_icon(&self) -> Result<()> {
        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.h_wnd,
            uID: self.internal_id,
            uFlags: NIF_ICON,
            ..Default::default()
        };

        if unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) }.into() {
            Ok(())
        } else {
            Err(fail_message(
                "Failed to remove notify icon (Shell_NotifyIconW)",
            ))
        }
    }
}

impl Drop for NotifyIconInternal {
    fn drop(&mut self) {
        self.remove_tray_icon()
            .expect("Notify icon should be removed");
        unsafe { DestroyWindow(self.h_wnd) }.expect("Window should be destroyed");
    }
}

/// Get the library instance handle.
/// Taken from https://github.com/rust-windowing/winit/blob/4e2e764e4a29305d612b7978b22583319c0458a0/src/platform_impl/windows/util.rs#L140
/// with slight modification.
fn get_instance_handle() -> HINSTANCE {
    // Gets the instance handle by taking the address of the
    // pseudo-variable created by the microsoft linker:
    // https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483

    // This is preferred over GetModuleHandle(NULL) because it also works in DLLs:
    // https://stackoverflow.com/questions/21718027/getmodulehandlenull-vs-hinstance

    extern "C" {
        static __ImageBase: IMAGE_DOS_HEADER;
    }

    HINSTANCE(&unsafe { __ImageBase } as *const _ as *mut _)
}

const fn loword(l: u32) -> u16 {
    l as u16
}

const fn hiword(l: u32) -> u16 {
    (l >> 16) as u16
}

const fn decode_coord(param: u32) -> PointInt32 {
    PointInt32 {
        X: loword(param) as i32,
        Y: hiword(param) as i32,
    }
}

extern "system" fn notify_proc(h_wnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if msg == WM_USER_TRAYICON {
        // temporary workaround for https://github.com/microsoft/win32metadata/issues/1765
        const NIN_SELECT: u32 = 1024;
        const NIN_KEYSELECT: u32 = 1025;

        let options = loword(l_param.0 as u32) as u32;
        if options == NIN_KEYSELECT || options == NIN_SELECT {
            INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                let internal = internal.as_mut().expect("global initialized");
                internal
                    .report_selected
                    .invoke_all(
                        &get_strong_ref(&internal.parent).expect("parent should stay valid"),
                        Some(&decode_coord(w_param.0 as u32)),
                    )
                    .expect("invoke_all should succeed");

                if let Some(ref menu) = internal.h_menu {
                    menu.show_menu(internal.h_wnd)
                        .expect("show_menu should succeed");
                }
            });

            return LRESULT(0);
        }
    }

    unsafe { DefWindowProcW(h_wnd, msg, w_param, l_param) }
}
