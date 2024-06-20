// Bindings generated by `windows-bindgen` 0.57.0

#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    dead_code,
    clippy::all
)]
windows_core::imp::define_interface!(
    ISequenceTranslator,
    ISequenceTranslator_Vtbl,
    0x340c58b0_ec80_5dc8_aa98_ca84d0528f74
);
impl windows_core::RuntimeType for ISequenceTranslator {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISequenceTranslator_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub BuildDictionary: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Translate: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        core::mem::MaybeUninit<windows_core::HSTRING>,
        *mut core::mem::MaybeUninit<windows_core::HSTRING>,
    ) -> windows_core::HRESULT,
}
#[repr(transparent)]
#[derive(PartialEq, Eq, core::fmt::Debug, Clone)]
pub struct SequenceTranslator(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SequenceTranslator,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl SequenceTranslator {
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
            SequenceTranslator,
            windows_core::imp::IGenericFactory,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    pub fn BuildDictionary(&self) -> windows_core::Result<()> {
        let this = self;
        unsafe {
            (windows_core::Interface::vtable(this).BuildDictionary)(
                windows_core::Interface::as_raw(this),
            )
            .ok()
        }
    }
    pub fn Translate(
        &self,
        input: &windows_core::HSTRING,
    ) -> windows_core::Result<windows_core::HSTRING> {
        let this = self;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Translate)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(input),
                &mut result__,
            )
            .and_then(|| windows_core::Type::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeType for SequenceTranslator {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISequenceTranslator>();
}
unsafe impl windows_core::Interface for SequenceTranslator {
    type Vtable = ISequenceTranslator_Vtbl;
    const IID: windows_core::GUID = <ISequenceTranslator as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SequenceTranslator {
    const NAME: &'static str = "LibSimbolMudah.SequenceTranslator";
}
unsafe impl Send for SequenceTranslator {}
unsafe impl Sync for SequenceTranslator {}
pub trait ISequenceTranslator_Impl: Sized {
    fn BuildDictionary(&self) -> windows_core::Result<()>;
    fn Translate(
        &self,
        input: &windows_core::HSTRING,
    ) -> windows_core::Result<windows_core::HSTRING>;
}
impl windows_core::RuntimeName for ISequenceTranslator {
    const NAME: &'static str = "LibSimbolMudah.ISequenceTranslator";
}
impl ISequenceTranslator_Vtbl {
    pub const fn new<
        Identity: windows_core::IUnknownImpl<Impl = Impl>,
        Impl: ISequenceTranslator_Impl,
        const OFFSET: isize,
    >() -> ISequenceTranslator_Vtbl {
        unsafe extern "system" fn BuildDictionary<
            Identity: windows_core::IUnknownImpl<Impl = Impl>,
            Impl: ISequenceTranslator_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            let this = (this as *const *const ()).offset(OFFSET) as *const Identity;
            let this = (*this).get_impl();
            ISequenceTranslator_Impl::BuildDictionary(this).into()
        }
        unsafe extern "system" fn Translate<
            Identity: windows_core::IUnknownImpl<Impl = Impl>,
            Impl: ISequenceTranslator_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            input: core::mem::MaybeUninit<windows_core::HSTRING>,
            result__: *mut core::mem::MaybeUninit<windows_core::HSTRING>,
        ) -> windows_core::HRESULT {
            let this = (this as *const *const ()).offset(OFFSET) as *const Identity;
            let this = (*this).get_impl();
            match ISequenceTranslator_Impl::Translate(this, core::mem::transmute(&input)) {
                Ok(ok__) => {
                    core::ptr::write(result__, core::mem::transmute_copy(&ok__));
                    core::mem::forget(ok__);
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, ISequenceTranslator, OFFSET>(),
            BuildDictionary: BuildDictionary::<Identity, Impl, OFFSET>,
            Translate: Translate::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ISequenceTranslator as windows_core::Interface>::IID
    }
}
