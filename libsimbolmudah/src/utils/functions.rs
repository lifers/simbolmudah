use std::{
    collections::HashSet,
    sync::{LazyLock, RwLock},
};

use windows::{
    core::{Error, Interface, Result, Weak, PCWSTR},
    Win32::{
        Foundation::{E_FAIL, E_POINTER, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, SystemServices::IMAGE_DOS_HEADER},
        UI::WindowsAndMessaging::{
            CreateWindowExW, RegisterClassW, CW_USEDEFAULT, HMENU, HWND_MESSAGE, WINDOW_EX_STYLE,
            WINDOW_STYLE, WNDCLASSW,
        },
    },
};

pub(crate) fn fail(error: impl std::error::Error) -> Error {
    Error::new(E_FAIL, format!("{:?}", error))
}

pub(crate) fn fail_message(message: &str) -> Error {
    Error::new(E_FAIL, message)
}

pub(crate) fn get_strong_ref<T>(weak: &Weak<T>) -> Result<T>
where
    T: Interface,
{
    weak.upgrade()
        .ok_or_else(|| Error::new(E_POINTER, "Weak pointer died"))
}

/// Get the library instance handle.
/// Taken from https://github.com/rust-windowing/winit/blob/4e2e764e4a29305d612b7978b22583319c0458a0/src/platform_impl/windows/util.rs#L140
/// with slight modification.
pub(crate) fn get_instance_handle() -> HINSTANCE {
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

/// Create a message-only window.
pub(crate) fn create_message_only_window(
    class_name: PCWSTR,
    wnd_proc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
) -> Result<HWND> {
    static REGISTERED: LazyLock<RwLock<HashSet<String>>> =
        LazyLock::new(|| RwLock::new(HashSet::new()));

    let h_instance = HINSTANCE(unsafe { GetModuleHandleW(None) }?.0);
    let string_name = unsafe { class_name.to_string() }?;

    if !REGISTERED.read().map_err(fail)?.contains(&string_name) {
        let wnd_class = WNDCLASSW {
            lpfnWndProc: wnd_proc,
            lpszClassName: class_name,
            hInstance: h_instance,
            ..Default::default()
        };

        if unsafe { RegisterClassW(&wnd_class) } == 0 {
            return Err(fail_message("RegisterClassW failed"));
        } else {
            REGISTERED.write().map_err(fail)?.insert(string_name);
        }
    }

    unsafe {
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
    }
}
