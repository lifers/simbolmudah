use windows::{
    core::{Error, Interface, Result, Weak},
    Win32::{
        Foundation::{E_FAIL, E_POINTER, HINSTANCE},
        System::SystemServices::IMAGE_DOS_HEADER,
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
