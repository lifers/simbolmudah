use windows::{
    core::{w, Owned, Result},
    Win32::{
        Foundation::{HWND, POINT},
        UI::WindowsAndMessaging::{
            CreatePopupMenu, GetCursorPos, InsertMenuW, SetForegroundWindow, SetMenuInfo,
            TrackPopupMenuEx, HMENU, MENUINFO, MF_BYPOSITION, MF_CHECKED, MF_SEPARATOR, MF_STRING,
            MF_UNCHECKED, MIM_STYLE, MNS_AUTODISMISS, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
            TPM_LEFTBUTTON,
        },
    },
};

pub(super) const WM_USER_SHOW_SETTINGS: usize = 0x1773;
pub(super) const WM_USER_LISTEN: usize = 0x1774;
pub(super) const WM_USER_EXIT: usize = 0x1775;

pub(super) struct NotifyIconMenu {
    h_menu: Owned<HMENU>,
    is_listening: bool,
}

impl NotifyIconMenu {
    pub(super) fn new(is_listening: bool) -> Result<Self> {
        unsafe {
            let h_menu = Owned::new(CreatePopupMenu()?);
            let res = Self {
                h_menu,
                is_listening,
            };

            let state = MF_BYPOSITION
                | MF_STRING
                | if is_listening {
                    MF_CHECKED
                } else {
                    MF_UNCHECKED
                };

            InsertMenuW(
                *res.h_menu,
                0,
                state,
                WM_USER_LISTEN,
                w!("Listen to compose"),
            )?;
            InsertMenuW(
                *res.h_menu,
                1,
                MF_BYPOSITION | MF_STRING,
                WM_USER_SHOW_SETTINGS,
                w!("Open settings"),
            )?;
            InsertMenuW(*res.h_menu, 2, MF_BYPOSITION | MF_SEPARATOR, 0, w!(""))?;
            InsertMenuW(
                *res.h_menu,
                3,
                MF_BYPOSITION | MF_STRING,
                WM_USER_EXIT,
                w!("Exit simbolmudah"),
            )?;

            Ok(res)
        }
    }

    pub(super) fn show_menu(&self, h_wnd: HWND) -> Result<()> {
        unsafe {
            SetForegroundWindow(h_wnd).ok()?;

            let mut point = POINT::default();
            GetCursorPos(&mut point)?;

            let menu_info = MENUINFO {
                cbSize: std::mem::size_of::<MENUINFO>() as u32,
                fMask: MIM_STYLE,
                dwStyle: MNS_AUTODISMISS,
                ..Default::default()
            };
            SetMenuInfo(*self.h_menu, &menu_info)?;

            TrackPopupMenuEx(
                *self.h_menu,
                (TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_LEFTBUTTON).0,
                point.x,
                point.y,
                h_wnd,
                None,
            )
            .ok()
        }
    }

    pub(super) fn is_listening(&self) -> bool {
        self.is_listening
    }
}
