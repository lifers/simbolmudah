use std::{
    collections::HashSet,
    sync::{LazyLock, RwLock},
};

use windows::{
    core::{Result, HSTRING, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, RegisterClassW, CW_USEDEFAULT, HMENU, HWND_MESSAGE,
            WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW,
        },
    },
};

use super::functions::{fail, fail_message};

/// Message-only window.
pub(crate) struct MessageWindow {
    h_wnd: HWND,
}

impl MessageWindow {
    pub(crate) fn new(
        class_name: &HSTRING,
        wnd_proc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
    ) -> Result<Self> {
        static REGISTERED: LazyLock<RwLock<HashSet<String>>> =
            LazyLock::new(|| RwLock::new(HashSet::new()));

        // Do not free this handle.
        let h_instance = unsafe { GetModuleHandleW(None) }?.into();
        let string_name = class_name.to_string();

        if !REGISTERED.read().map_err(fail)?.contains(&string_name) {
            let wnd_class = WNDCLASSW {
                lpfnWndProc: wnd_proc,
                lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
                hInstance: h_instance,
                ..Default::default()
            };

            if unsafe { RegisterClassW(&wnd_class) } == 0 {
                return Err(fail_message("RegisterClassW failed"));
            } else {
                REGISTERED.write().map_err(fail)?.insert(string_name);
            }
        }

        Ok(Self {
            h_wnd: unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    class_name,
                    PCWSTR::null(),
                    WINDOW_STYLE::default(),
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    HWND_MESSAGE,
                    HMENU::default(),
                    h_instance,
                    None,
                )
            }?,
        })
    }

    pub(crate) fn handle(&self) -> HWND {
        self.h_wnd
    }
}

impl Drop for MessageWindow {
    fn drop(&mut self) {
        unsafe { DestroyWindow(self.h_wnd) }.expect("Window should be destroyed");
    }
}
