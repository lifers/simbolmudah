/// Taken from https://github.com/tauri-apps/tray-icon/blob/d4078696edba67b0ab42cef67e6a421a0332c96f/src/platform_impl/windows/mod.rs
/// with modifications.
use std::cell::RefCell;

use windows::{
    core::{w, Result, Weak},
    Foundation::EventRegistrationToken,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD,
                NIM_DELETE, NIM_MODIFY, NIM_SETVERSION, NOTIFYICONDATAW, NOTIFYICONDATAW_0,
                NOTIFYICON_VERSION_4,
            },
            WindowsAndMessaging::{
                DefWindowProcW, LoadIconW, PostQuitMessage, HICON, IDI_WARNING, WM_COMMAND,
                WM_DESTROY,
            },
        },
    },
};

use crate::{
    bindings,
    utils::{
        delegate_storage::DelegateStorage,
        functions::{fail_message, get_strong_ref},
        message_window::MessageWindow,
    },
};

use super::{
    counter::GLOBAL_COUNTER,
    menu::{NotifyIconMenu, WM_USER_EXIT, WM_USER_LISTEN, WM_USER_SHOW_SETTINGS},
};

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
    h_wnd: MessageWindow,
    internal_id: u32,
    h_icon: HICON,
    h_menu: Option<NotifyIconMenu>,
    pub(super) report_open_settings: DelegateStorage<bindings::NotifyIcon, bool>,
    pub(super) report_exit_app: DelegateStorage<bindings::NotifyIcon, bool>,
    pub(super) report_set_listening: DelegateStorage<bindings::NotifyIcon, bool>,
    pub(super) on_state_changed_token: EventRegistrationToken,
    pub(super) parent: Weak<bindings::NotifyIcon>,
}

impl NotifyIconInternal {
    pub(super) fn create_for_thread(
        parent: Weak<bindings::NotifyIcon>,
        hookenabled: bool,
    ) -> Result<()> {
        let res = Self::new(hookenabled, parent)?;
        INTERNAL_NOTIFYICON.set(Some(res));
        Ok(())
    }

    fn new(listening: bool, parent: Weak<bindings::NotifyIcon>) -> Result<Self> {
        let internal_id = GLOBAL_COUNTER.next();
        let h_wnd = MessageWindow::new(w!("LibSimbolMudah.NotifyIcon"), Some(notify_proc))?;
        let h_menu = NotifyIconMenu::new(listening)?;
        let h_icon = unsafe { LoadIconW(None, IDI_WARNING) }?;

        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: h_wnd.handle(),
            uID: internal_id,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: WM_USER_TRAYICON,
            hIcon: h_icon,
            szTip: get_tooltip(listening),
            Anonymous: NOTIFYICONDATAW_0 {
                uVersion: NOTIFYICON_VERSION_4,
            },
            ..Default::default()
        };

        unsafe { Shell_NotifyIconW(NIM_ADD, &nid) }.ok()?;
        unsafe { Shell_NotifyIconW(NIM_SETVERSION, &nid) }.ok()?;

        Ok(Self {
            h_wnd,
            internal_id,
            h_icon,
            h_menu: Some(h_menu),
            report_open_settings: DelegateStorage::new(),
            report_exit_app: DelegateStorage::new(),
            report_set_listening: DelegateStorage::new(),
            on_state_changed_token: EventRegistrationToken::default(),
            parent,
        })
    }

    fn update_notify_icon(&self, listening: bool) -> Result<()> {
        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.h_wnd.handle(),
            uID: self.internal_id,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: WM_USER_TRAYICON,
            hIcon: self.h_icon,
            szTip: get_tooltip(listening),
            Anonymous: NOTIFYICONDATAW_0 {
                uVersion: NOTIFYICON_VERSION_4,
            },
            ..Default::default()
        };

        if unsafe { Shell_NotifyIconW(NIM_MODIFY, &nid) }.into() {
            Ok(())
        } else {
            Err(fail_message(
                "Failed to update notify icon (Shell_NotifyIconW)",
            ))
        }
    }

    fn remove_tray_icon(&self) -> Result<()> {
        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.h_wnd.handle(),
            uID: self.internal_id,
            uFlags: NIF_ICON,
            ..Default::default()
        };

        unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) }.ok()
    }

    pub(super) fn update_listening_check(&mut self, listening: bool) -> Result<()> {
        self.h_menu = Some(NotifyIconMenu::new(listening)?);
        self.update_notify_icon(listening)?;
        Ok(())
    }
}

impl Drop for NotifyIconInternal {
    fn drop(&mut self) {
        self.remove_tray_icon()
            .expect("Notify icon should be removed");
    }
}

fn get_tooltip(listening: bool) -> [u16; 128] {
    let mut sz_tip = [0; 128];
    let tip = if listening {
        "simbolmudah (listening)"
    } else {
        "simbolmudah (not listening)"
    }
    .encode_utf16()
    .collect::<Box<_>>();
    sz_tip[..tip.len().min(128)].copy_from_slice(&tip[..tip.len().min(128)]);
    sz_tip
}

extern "system" fn notify_proc(h_wnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        WM_USER_TRAYICON => {
            // temporary workaround for https://github.com/microsoft/win32metadata/issues/1765
            const NIN_SELECT: u32 = 1024;
            const NIN_KEYSELECT: u32 = 1025;

            let options = (l_param.0 as u16).into();
            if matches!(options, NIN_SELECT | NIN_KEYSELECT) {
                INTERNAL_NOTIFYICON.with_borrow(|internal: &Option<NotifyIconInternal>| {
                    let internal = internal.as_ref().expect("global initialized");

                    if let Some(ref menu) = internal.h_menu {
                        menu.show_menu(internal.h_wnd.handle())
                            .expect("show_menu should succeed");
                    }
                });
            }
        }
        WM_DESTROY => unsafe { PostQuitMessage(0) },
        WM_COMMAND => match (w_param.0 as u16).into() {
            WM_USER_SHOW_SETTINGS => {
                INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                    let internal = internal.as_mut().expect("global initialized");
                    internal
                        .report_open_settings
                        .invoke_all(
                            &get_strong_ref(&internal.parent).expect("parent should stay valid"),
                            None,
                        )
                        .expect("invoke_all should succeed");
                });
            }
            WM_USER_LISTEN => {
                INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                    let internal = internal.as_mut().expect("global initialized");
                    internal
                        .report_set_listening
                        .invoke_all(
                            &get_strong_ref(&internal.parent).expect("parent should stay valid"),
                            Some(
                                &!internal
                                    .h_menu
                                    .as_ref()
                                    .expect("menu should stay valid")
                                    .is_listening(),
                            ),
                        )
                        .expect("invoke_all should succeed");
                });
            }
            WM_USER_EXIT => {
                INTERNAL_NOTIFYICON.with_borrow_mut(|internal: &mut Option<NotifyIconInternal>| {
                    let internal = internal.as_mut().expect("global initialized");
                    internal
                        .report_exit_app
                        .invoke_all(
                            &get_strong_ref(&internal.parent).expect("parent should stay valid"),
                            None,
                        )
                        .expect("invoke_all should succeed");
                });
            }
            _ => {}
        },
        _ => {}
    }

    unsafe { DefWindowProcW(h_wnd, msg, w_param, l_param) }
}
