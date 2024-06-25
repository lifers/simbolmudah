// Bindings generated by `windows-bindgen` 0.57.0

#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    dead_code,
    clippy::all
)]
windows_core::imp::define_interface!(
    IKeyboardTranslator,
    IKeyboardTranslator_Vtbl,
    0x30b37850_7ffb_54e1_a40e_fa6cbcaba2ed
);
impl windows_core::RuntimeType for IKeyboardTranslator {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IKeyboardTranslator_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub TranslateAndForward: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        bool,
        bool,
        bool,
        u8,
    ) -> windows_core::HRESULT,
    pub CheckLayoutAndUpdate:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub BuildTranslator: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub OnTranslated: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows::Foundation::EventRegistrationToken,
    ) -> windows_core::HRESULT,
    pub RemoveOnTranslated: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows::Foundation::EventRegistrationToken,
    ) -> windows_core::HRESULT,
    pub OnInvalid: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows::Foundation::EventRegistrationToken,
    ) -> windows_core::HRESULT,
    pub RemoveOnInvalid: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows::Foundation::EventRegistrationToken,
    ) -> windows_core::HRESULT,
}
#[repr(transparent)]
#[derive(PartialEq, Eq, core::fmt::Debug, Clone)]
pub struct KeyboardTranslator(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    KeyboardTranslator,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl KeyboardTranslator {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<
        R,
        F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            KeyboardTranslator,
            windows_core::imp::IGenericFactory,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    pub fn TranslateAndForward(
        &self,
        vkcode: u32,
        scancode: u32,
        hascapslock: bool,
        hasshift: bool,
        hasaltgr: bool,
        destination: u8,
    ) -> windows_core::Result<()> {
        let this = self;
        unsafe {
            (windows_core::Interface::vtable(this).TranslateAndForward)(
                windows_core::Interface::as_raw(this),
                vkcode,
                scancode,
                hascapslock,
                hasshift,
                hasaltgr,
                destination,
            )
            .ok()
        }
    }
    pub fn CheckLayoutAndUpdate(&self) -> windows_core::Result<()> {
        let this = self;
        unsafe {
            (windows_core::Interface::vtable(this).CheckLayoutAndUpdate)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub fn BuildTranslator(&self) -> windows_core::Result<()> {
        let this = self;
        unsafe {
            (windows_core::Interface::vtable(this).BuildTranslator)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub fn OnTranslated<P0>(
        &self,
        handler: P0,
    ) -> windows_core::Result<windows::Foundation::EventRegistrationToken>
    where
        P0: windows_core::Param<
            windows::Foundation::TypedEventHandler<KeyboardTranslator, windows_core::HSTRING>,
        >,
    {
        let this = self;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).OnTranslated)(
                windows_core::Interface::as_raw(this),
                handler.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn RemoveOnTranslated(
        &self,
        token: windows::Foundation::EventRegistrationToken,
    ) -> windows_core::Result<()> {
        let this = self;
        unsafe {
            (windows_core::Interface::vtable(this).RemoveOnTranslated)(
                windows_core::Interface::as_raw(this),
                token,
            )
            .ok()
        }
    }
    pub fn OnInvalid<P0>(
        &self,
        handler: P0,
    ) -> windows_core::Result<windows::Foundation::EventRegistrationToken>
    where
        P0: windows_core::Param<
            windows::Foundation::TypedEventHandler<KeyboardTranslator, windows_core::HSTRING>,
        >,
    {
        let this = self;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).OnInvalid)(
                windows_core::Interface::as_raw(this),
                handler.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn RemoveOnInvalid(
        &self,
        token: windows::Foundation::EventRegistrationToken,
    ) -> windows_core::Result<()> {
        let this = self;
        unsafe {
            (windows_core::Interface::vtable(this).RemoveOnInvalid)(
                windows_core::Interface::as_raw(this),
                token,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for KeyboardTranslator {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IKeyboardTranslator>();
}
unsafe impl windows_core::Interface for KeyboardTranslator {
    type Vtable = IKeyboardTranslator_Vtbl;
    const IID: windows_core::GUID = <IKeyboardTranslator as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for KeyboardTranslator {
    const NAME: &'static str = "LibSimbolMudah.KeyboardTranslator";
}
unsafe impl Send for KeyboardTranslator {}
unsafe impl Sync for KeyboardTranslator {}
pub trait IKeyboardTranslator_Impl: Sized {
    fn TranslateAndForward(
        &self,
        vkcode: u32,
        scancode: u32,
        hascapslock: bool,
        hasshift: bool,
        hasaltgr: bool,
        destination: u8,
    ) -> windows_core::Result<()>;
    fn CheckLayoutAndUpdate(&self) -> windows_core::Result<()>;
    fn BuildTranslator(&self) -> windows_core::Result<()>;
    fn OnTranslated(
        &self,
        handler: Option<
            &windows::Foundation::TypedEventHandler<KeyboardTranslator, windows_core::HSTRING>,
        >,
    ) -> windows_core::Result<windows::Foundation::EventRegistrationToken>;
    fn RemoveOnTranslated(
        &self,
        token: &windows::Foundation::EventRegistrationToken,
    ) -> windows_core::Result<()>;
    fn OnInvalid(
        &self,
        handler: Option<
            &windows::Foundation::TypedEventHandler<KeyboardTranslator, windows_core::HSTRING>,
        >,
    ) -> windows_core::Result<windows::Foundation::EventRegistrationToken>;
    fn RemoveOnInvalid(
        &self,
        token: &windows::Foundation::EventRegistrationToken,
    ) -> windows_core::Result<()>;
}
impl windows_core::RuntimeName for IKeyboardTranslator {
    const NAME: &'static str = "LibSimbolMudah.IKeyboardTranslator";
}
impl IKeyboardTranslator_Vtbl {
    pub const fn new<
        Identity: windows_core::IUnknownImpl<Impl = Impl>,
        Impl: IKeyboardTranslator_Impl,
        const OFFSET: isize,
    >() -> IKeyboardTranslator_Vtbl {
        unsafe extern "system" fn TranslateAndForward<
            Identity: windows_core::IUnknownImpl<Impl = Impl>,
            Impl: IKeyboardTranslator_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            vkcode: u32,
            scancode: u32,
            hascapslock: bool,
            hasshift: bool,
            hasaltgr: bool,
            destination: u8,
        ) -> windows_core::HRESULT {
            let this = (this as *const *const ()).offset(OFFSET) as *const Identity;
            let this = (*this).get_impl();
            IKeyboardTranslator_Impl::TranslateAndForward(
                this,
                vkcode,
                scancode,
                hascapslock,
                hasshift,
                hasaltgr,
                destination,
            )
            .into()
        }
        unsafe extern "system" fn CheckLayoutAndUpdate<
            Identity: windows_core::IUnknownImpl<Impl = Impl>,
            Impl: IKeyboardTranslator_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            let this = (this as *const *const ()).offset(OFFSET) as *const Identity;
            let this = (*this).get_impl();
            IKeyboardTranslator_Impl::CheckLayoutAndUpdate(this).into()
        }
        unsafe extern "system" fn BuildTranslator<
            Identity: windows_core::IUnknownImpl<Impl = Impl>,
            Impl: IKeyboardTranslator_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            let this = (this as *const *const ()).offset(OFFSET) as *const Identity;
            let this = (*this).get_impl();
            IKeyboardTranslator_Impl::BuildTranslator(this).into()
        }
        unsafe extern "system" fn OnTranslated<
            Identity: windows_core::IUnknownImpl<Impl = Impl>,
            Impl: IKeyboardTranslator_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            result__: *mut windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT {
            let this = (this as *const *const ()).offset(OFFSET) as *const Identity;
            let this = (*this).get_impl();
            match IKeyboardTranslator_Impl::OnTranslated(
                this,
                windows_core::from_raw_borrowed(&handler),
            ) {
                Ok(ok__) => {
                    core::ptr::write(result__, core::mem::transmute_copy(&ok__));
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn RemoveOnTranslated<
            Identity: windows_core::IUnknownImpl<Impl = Impl>,
            Impl: IKeyboardTranslator_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            token: windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT {
            let this = (this as *const *const ()).offset(OFFSET) as *const Identity;
            let this = (*this).get_impl();
            IKeyboardTranslator_Impl::RemoveOnTranslated(this, core::mem::transmute(&token)).into()
        }
        unsafe extern "system" fn OnInvalid<
            Identity: windows_core::IUnknownImpl<Impl = Impl>,
            Impl: IKeyboardTranslator_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            result__: *mut windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT {
            let this = (this as *const *const ()).offset(OFFSET) as *const Identity;
            let this = (*this).get_impl();
            match IKeyboardTranslator_Impl::OnInvalid(
                this,
                windows_core::from_raw_borrowed(&handler),
            ) {
                Ok(ok__) => {
                    core::ptr::write(result__, core::mem::transmute_copy(&ok__));
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn RemoveOnInvalid<
            Identity: windows_core::IUnknownImpl<Impl = Impl>,
            Impl: IKeyboardTranslator_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            token: windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT {
            let this = (this as *const *const ()).offset(OFFSET) as *const Identity;
            let this = (*this).get_impl();
            IKeyboardTranslator_Impl::RemoveOnInvalid(this, core::mem::transmute(&token)).into()
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IKeyboardTranslator, OFFSET>(),
            TranslateAndForward: TranslateAndForward::<Identity, Impl, OFFSET>,
            CheckLayoutAndUpdate: CheckLayoutAndUpdate::<Identity, Impl, OFFSET>,
            BuildTranslator: BuildTranslator::<Identity, Impl, OFFSET>,
            OnTranslated: OnTranslated::<Identity, Impl, OFFSET>,
            RemoveOnTranslated: RemoveOnTranslated::<Identity, Impl, OFFSET>,
            OnInvalid: OnInvalid::<Identity, Impl, OFFSET>,
            RemoveOnInvalid: RemoveOnInvalid::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IKeyboardTranslator as windows_core::Interface>::IID
    }
}