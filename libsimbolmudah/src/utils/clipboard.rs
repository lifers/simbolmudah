use windows::{
    core::{w, Error, Result, HRESULT},
    Win32::{
        Foundation::{HANDLE, HGLOBAL, HWND},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW,
                SetClipboardData,
            },
            Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
            Ole::CF_UNICODETEXT,
        },
    },
};

pub(super) struct Clipboard;

impl Clipboard {
    pub(super) fn new(owner_wnd: HWND) -> Result<Self> {
        // Safety: if OpenClipboard succeds but the next functions fail, the clipboard will be properly closed
        // through the Drop implementation.
        unsafe {
            OpenClipboard(owner_wnd)?;
            let res = Self;
            EmptyClipboard()?;
            exclude_clipboard_content_from_monitor_processing()?;
            Ok(res)
        }
    }

    pub(super) fn set_text(&self, text: &str) -> Result<()> {
        let u_text = text.encode_utf16().collect::<Box<_>>();
        unsafe {
            let h_global = GlobalAlloc(GMEM_MOVEABLE, size_of::<u16>() * (u_text.len() + 1))?;
            let h_ptr = GlobalLock(h_global) as *mut u16;
            h_ptr.copy_from_nonoverlapping(u_text.as_ptr(), u_text.len());
            h_ptr.offset(u_text.len() as isize).write(0);

            global_unlock(h_global)?;
            let _ = SetClipboardData(CF_UNICODETEXT.0.into(), HANDLE(h_global.0))?;
        }
        Ok(())
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe { CloseClipboard() }.expect("clipboard should be properly closed");
    }
}

/// Compensate for GlobalUnlock's weird return policy.
/// See https://github.com/microsoft/win32metadata/issues/1770
unsafe fn global_unlock(h_global: HGLOBAL) -> Result<()> {
    if let Err(e) = GlobalUnlock(h_global) {
        match e.code() {
            HRESULT(0) => Ok(()),
            _ => Err(Error::from_hresult(e.code())),
        }
    } else {
        Ok(())
    }
}

/// Exclude clipboard content from clipboard history and cloud clipboard.
/// https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats#cloud-clipboard-and-clipboard-history-formats
unsafe fn exclude_clipboard_content_from_monitor_processing() -> Result<()> {
    let x = RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"));
    let h_global = GlobalAlloc(GMEM_MOVEABLE, size_of::<u32>())?;
    let h_ptr = GlobalLock(h_global) as *mut u32;
    h_ptr.write(0);
    global_unlock(h_global)?;
    let _ = SetClipboardData(x, HANDLE(h_global.0))?;
    Ok(())
}
