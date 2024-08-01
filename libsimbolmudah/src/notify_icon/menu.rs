use windows::{
    core::{w, Result},
    Win32::{
        Foundation::{HWND, POINT},
        UI::WindowsAndMessaging::{
            CreatePopupMenu, DestroyMenu, GetCursorPos, InsertMenuW, SetForegroundWindow,
            SetMenuInfo, TrackPopupMenuEx, HMENU, MENUINFO, MF_BYPOSITION, MF_CHECKED,
            MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MIM_STYLE, MNS_AUTODISMISS, TPM_BOTTOMALIGN,
            TPM_LEFTALIGN, TPM_LEFTBUTTON,
        },
    },
};

use crate::fail_message;

pub(super) const WM_USER_SHOW_SETTINGS: usize = 0x1773;
pub(super) const WM_USER_LISTEN: usize = 0x1774;

pub(super) struct NotifyIconMenu {
    h_menu: HMENU,
    is_listening: bool,
}

impl NotifyIconMenu {
    pub(super) fn new(is_listening: bool) -> Result<Self> {
        let h_menu = unsafe { CreatePopupMenu() }?;

        unsafe {
            InsertMenuW(
                h_menu,
                0,
                MF_BYPOSITION | MF_STRING,
                WM_USER_SHOW_SETTINGS,
                w!("Open settings"),
            )
        }?;
        unsafe { InsertMenuW(h_menu, 1, MF_BYPOSITION | MF_SEPARATOR, 0, w!("")) }?;

        let state = MF_BYPOSITION | MF_STRING | if is_listening { MF_CHECKED } else { MF_UNCHECKED };
        unsafe { InsertMenuW(h_menu, 0, state, WM_USER_LISTEN, w!("Listen to compose")) }?;
        Ok(Self { h_menu, is_listening })
    }

    pub(super) fn show_menu(&self, h_wnd: HWND) -> Result<()> {
        if !unsafe { SetForegroundWindow(h_wnd) }.as_bool() {
            return Err(fail_message("Failed to set foreground window"));
        }

        let mut point = POINT::default();
        unsafe { GetCursorPos(&mut point) }?;

        let menu_info = MENUINFO {
            cbSize: std::mem::size_of::<MENUINFO>() as u32,
            fMask: MIM_STYLE,
            dwStyle: MNS_AUTODISMISS,
            ..Default::default()
        };
        unsafe { SetMenuInfo(self.h_menu, &menu_info) }?;

        if !unsafe {
            TrackPopupMenuEx(
                self.h_menu,
                (TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_LEFTBUTTON).0,
                point.x,
                point.y,
                h_wnd,
                None,
            )
        }
        .as_bool()
        {
            return Err(fail_message("Failed to show menu"));
        }

        Ok(())
    }

    pub(super) fn is_listening(&self) -> bool {
        self.is_listening
    }
}

impl Drop for NotifyIconMenu {
    fn drop(&mut self) {
        unsafe { DestroyMenu(self.h_menu) }.expect("menu must be destroyed");
    }
}
