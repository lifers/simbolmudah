// Bindings generated by `windows-bindgen` 0.58.0

#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    dead_code,
    clippy::all
)]
windows_core::imp::define_interface!(
    IKeyboardHook,
    IKeyboardHook_Vtbl,
    0xa36838bf_852a_5d0f_a772_76cada53be3f
);
impl windows_core::RuntimeType for IKeyboardHook {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IKeyboardHook_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub DebugStateChanged: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows::Foundation::EventRegistrationToken,
    ) -> windows_core::HRESULT,
    pub RemoveDebugStateChanged: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows::Foundation::EventRegistrationToken,
    ) -> windows_core::HRESULT,
    pub DebugKeyEvent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows::Foundation::EventRegistrationToken,
    ) -> windows_core::HRESULT,
    pub RemoveDebugKeyEvent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows::Foundation::EventRegistrationToken,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IKeyboardHookFactory,
    IKeyboardHookFactory_Vtbl,
    0xed7819e6_f21e_54c9_ab1c_eeb9de4c18f2
);
impl windows_core::RuntimeType for IKeyboardHookFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IKeyboardHookFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateInstance: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IKeyboardTranslator,
    IKeyboardTranslator_Vtbl,
    0x459f4272_0cb3_5e0f_b071_9ab3f095737d
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
windows_core::imp::define_interface!(
    IKeyboardTranslatorFactory,
    IKeyboardTranslatorFactory_Vtbl,
    0x53b7cb66_7ff7_506c_a68b_ee25ddaf9709
);
impl windows_core::RuntimeType for IKeyboardTranslatorFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IKeyboardTranslatorFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateInstance: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ISequenceDefinition,
    ISequenceDefinition_Vtbl,
    0x69655d23_79eb_5c1a_8585_f3e345fe8293
);
impl windows_core::RuntimeType for ISequenceDefinition {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISequenceDefinition_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Build: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        core::mem::MaybeUninit<windows_core::HSTRING>,
        core::mem::MaybeUninit<windows_core::HSTRING>,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ISequenceSearcher,
    ISequenceSearcher_Vtbl,
    0x9881b990_042e_50ad_84e0_d2c1eb4a79e2
);
impl windows_core::RuntimeType for ISequenceSearcher {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISequenceSearcher_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Search: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        core::mem::MaybeUninit<windows_core::HSTRING>,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ISequenceSearcherFactory,
    ISequenceSearcherFactory_Vtbl,
    0xa45320a9_3119_50d9_ba29_b37a2846a1cb
);
impl windows_core::RuntimeType for ISequenceSearcherFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct ISequenceSearcherFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateInstance: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
#[repr(transparent)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct KeyboardHook(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    KeyboardHook,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl KeyboardHook {
    pub fn DebugStateChanged<P0>(
        &self,
        handler: P0,
    ) -> windows_core::Result<windows::Foundation::EventRegistrationToken>
    where
        P0: windows_core::Param<
            windows::Foundation::TypedEventHandler<KeyboardHook, windows_core::HSTRING>,
        >,
    {
        let this = self;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DebugStateChanged)(
                windows_core::Interface::as_raw(this),
                handler.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn RemoveDebugStateChanged(
        &self,
        token: windows::Foundation::EventRegistrationToken,
    ) -> windows_core::Result<()> {
        let this = self;
        unsafe {
            (windows_core::Interface::vtable(this).RemoveDebugStateChanged)(
                windows_core::Interface::as_raw(this),
                token,
            )
            .ok()
        }
    }
    pub fn DebugKeyEvent<P0>(
        &self,
        handler: P0,
    ) -> windows_core::Result<windows::Foundation::EventRegistrationToken>
    where
        P0: windows_core::Param<
            windows::Foundation::TypedEventHandler<KeyboardHook, windows_core::HSTRING>,
        >,
    {
        let this = self;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DebugKeyEvent)(
                windows_core::Interface::as_raw(this),
                handler.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn RemoveDebugKeyEvent(
        &self,
        token: windows::Foundation::EventRegistrationToken,
    ) -> windows_core::Result<()> {
        let this = self;
        unsafe {
            (windows_core::Interface::vtable(this).RemoveDebugKeyEvent)(
                windows_core::Interface::as_raw(this),
                token,
            )
            .ok()
        }
    }
    pub fn CreateInstance<P0>(translator: P0) -> windows_core::Result<KeyboardHook>
    where
        P0: windows_core::Param<KeyboardTranslator>,
    {
        Self::IKeyboardHookFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInstance)(
                windows_core::Interface::as_raw(this),
                translator.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::Type::from_abi(result__))
        })
    }
    #[doc(hidden)]
    pub fn IKeyboardHookFactory<R, F: FnOnce(&IKeyboardHookFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<KeyboardHook, IKeyboardHookFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for KeyboardHook {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IKeyboardHook>();
}
unsafe impl windows_core::Interface for KeyboardHook {
    type Vtable = IKeyboardHook_Vtbl;
    const IID: windows_core::GUID = <IKeyboardHook as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for KeyboardHook {
    const NAME: &'static str = "LibSimbolMudah.KeyboardHook";
}
unsafe impl Send for KeyboardHook {}
unsafe impl Sync for KeyboardHook {}
#[repr(transparent)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct KeyboardTranslator(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    KeyboardTranslator,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl KeyboardTranslator {
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
    pub fn CreateInstance<P0>(definition: P0) -> windows_core::Result<KeyboardTranslator>
    where
        P0: windows_core::Param<SequenceDefinition>,
    {
        Self::IKeyboardTranslatorFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInstance)(
                windows_core::Interface::as_raw(this),
                definition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::Type::from_abi(result__))
        })
    }
    #[doc(hidden)]
    pub fn IKeyboardTranslatorFactory<
        R,
        F: FnOnce(&IKeyboardTranslatorFactory) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            KeyboardTranslator,
            IKeyboardTranslatorFactory,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
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
#[repr(transparent)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SequenceDefinition(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SequenceDefinition,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl SequenceDefinition {
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
            SequenceDefinition,
            windows_core::imp::IGenericFactory,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    pub fn Build(
        &self,
        keysymdef: &windows_core::HSTRING,
        composedef: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = self;
        unsafe {
            (windows_core::Interface::vtable(this).Build)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(keysymdef),
                core::mem::transmute_copy(composedef),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for SequenceDefinition {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISequenceDefinition>();
}
unsafe impl windows_core::Interface for SequenceDefinition {
    type Vtable = ISequenceDefinition_Vtbl;
    const IID: windows_core::GUID = <ISequenceDefinition as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SequenceDefinition {
    const NAME: &'static str = "LibSimbolMudah.SequenceDefinition";
}
unsafe impl Send for SequenceDefinition {}
unsafe impl Sync for SequenceDefinition {}
#[repr(transparent)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SequenceSearcher(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    SequenceSearcher,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl SequenceSearcher {
    pub fn Search(
        &self,
        keyword: &windows_core::HSTRING,
    ) -> windows_core::Result<windows::Foundation::Collections::IVectorView<SequenceDescription>>
    {
        let this = self;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Search)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(keyword),
                &mut result__,
            )
            .and_then(|| windows_core::Type::from_abi(result__))
        }
    }
    pub fn CreateInstance<P0>(definition: P0) -> windows_core::Result<SequenceSearcher>
    where
        P0: windows_core::Param<SequenceDefinition>,
    {
        Self::ISequenceSearcherFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateInstance)(
                windows_core::Interface::as_raw(this),
                definition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::Type::from_abi(result__))
        })
    }
    #[doc(hidden)]
    pub fn ISequenceSearcherFactory<
        R,
        F: FnOnce(&ISequenceSearcherFactory) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<SequenceSearcher, ISequenceSearcherFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for SequenceSearcher {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ISequenceSearcher>();
}
unsafe impl windows_core::Interface for SequenceSearcher {
    type Vtable = ISequenceSearcher_Vtbl;
    const IID: windows_core::GUID = <ISequenceSearcher as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for SequenceSearcher {
    const NAME: &'static str = "LibSimbolMudah.SequenceSearcher";
}
unsafe impl Send for SequenceSearcher {}
unsafe impl Sync for SequenceSearcher {}
#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceDescription {
    pub sequence: windows_core::HSTRING,
    pub result: windows_core::HSTRING,
    pub description: windows_core::HSTRING,
}
impl windows_core::TypeKind for SequenceDescription {
    type TypeKind = windows_core::CloneType;
}
impl windows_core::RuntimeType for SequenceDescription {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"struct(LibSimbolMudah.SequenceDescription;string;string;string)",
    );
}
impl Default for SequenceDescription {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub trait IKeyboardHook_Impl: Sized {
    fn DebugStateChanged(
        &self,
        handler: Option<
            &windows::Foundation::TypedEventHandler<KeyboardHook, windows_core::HSTRING>,
        >,
    ) -> windows_core::Result<windows::Foundation::EventRegistrationToken>;
    fn RemoveDebugStateChanged(
        &self,
        token: &windows::Foundation::EventRegistrationToken,
    ) -> windows_core::Result<()>;
    fn DebugKeyEvent(
        &self,
        handler: Option<
            &windows::Foundation::TypedEventHandler<KeyboardHook, windows_core::HSTRING>,
        >,
    ) -> windows_core::Result<windows::Foundation::EventRegistrationToken>;
    fn RemoveDebugKeyEvent(
        &self,
        token: &windows::Foundation::EventRegistrationToken,
    ) -> windows_core::Result<()>;
}
impl windows_core::RuntimeName for IKeyboardHook {
    const NAME: &'static str = "LibSimbolMudah.IKeyboardHook";
}
impl IKeyboardHook_Vtbl {
    pub const fn new<Identity: windows_core::IUnknownImpl, const OFFSET: isize>(
    ) -> IKeyboardHook_Vtbl
    where
        Identity: IKeyboardHook_Impl,
    {
        unsafe extern "system" fn DebugStateChanged<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            result__: *mut windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardHook_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            match IKeyboardHook_Impl::DebugStateChanged(
                this,
                windows_core::from_raw_borrowed(&handler),
            ) {
                Ok(ok__) => {
                    result__.write(core::mem::transmute_copy(&ok__));
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn RemoveDebugStateChanged<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            token: windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardHook_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            IKeyboardHook_Impl::RemoveDebugStateChanged(this, core::mem::transmute(&token)).into()
        }
        unsafe extern "system" fn DebugKeyEvent<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            result__: *mut windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardHook_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            match IKeyboardHook_Impl::DebugKeyEvent(this, windows_core::from_raw_borrowed(&handler))
            {
                Ok(ok__) => {
                    result__.write(core::mem::transmute_copy(&ok__));
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn RemoveDebugKeyEvent<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            token: windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardHook_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            IKeyboardHook_Impl::RemoveDebugKeyEvent(this, core::mem::transmute(&token)).into()
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IKeyboardHook, OFFSET>(),
            DebugStateChanged: DebugStateChanged::<Identity, OFFSET>,
            RemoveDebugStateChanged: RemoveDebugStateChanged::<Identity, OFFSET>,
            DebugKeyEvent: DebugKeyEvent::<Identity, OFFSET>,
            RemoveDebugKeyEvent: RemoveDebugKeyEvent::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IKeyboardHook as windows_core::Interface>::IID
    }
}
pub trait IKeyboardHookFactory_Impl: Sized {
    fn CreateInstance(
        &self,
        translator: Option<&KeyboardTranslator>,
    ) -> windows_core::Result<KeyboardHook>;
}
impl windows_core::RuntimeName for IKeyboardHookFactory {
    const NAME: &'static str = "LibSimbolMudah.IKeyboardHookFactory";
}
impl IKeyboardHookFactory_Vtbl {
    pub const fn new<Identity: windows_core::IUnknownImpl, const OFFSET: isize>(
    ) -> IKeyboardHookFactory_Vtbl
    where
        Identity: IKeyboardHookFactory_Impl,
    {
        unsafe extern "system" fn CreateInstance<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            translator: *mut core::ffi::c_void,
            result__: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardHookFactory_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            match IKeyboardHookFactory_Impl::CreateInstance(
                this,
                windows_core::from_raw_borrowed(&translator),
            ) {
                Ok(ok__) => {
                    result__.write(core::mem::transmute_copy(&ok__));
                    core::mem::forget(ok__);
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IKeyboardHookFactory, OFFSET>(
            ),
            CreateInstance: CreateInstance::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IKeyboardHookFactory as windows_core::Interface>::IID
    }
}
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
    pub const fn new<Identity: windows_core::IUnknownImpl, const OFFSET: isize>(
    ) -> IKeyboardTranslator_Vtbl
    where
        Identity: IKeyboardTranslator_Impl,
    {
        unsafe extern "system" fn TranslateAndForward<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            vkcode: u32,
            scancode: u32,
            hascapslock: bool,
            hasshift: bool,
            hasaltgr: bool,
            destination: u8,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardTranslator_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
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
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardTranslator_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            IKeyboardTranslator_Impl::CheckLayoutAndUpdate(this).into()
        }
        unsafe extern "system" fn OnTranslated<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            result__: *mut windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardTranslator_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            match IKeyboardTranslator_Impl::OnTranslated(
                this,
                windows_core::from_raw_borrowed(&handler),
            ) {
                Ok(ok__) => {
                    result__.write(core::mem::transmute_copy(&ok__));
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn RemoveOnTranslated<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            token: windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardTranslator_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            IKeyboardTranslator_Impl::RemoveOnTranslated(this, core::mem::transmute(&token)).into()
        }
        unsafe extern "system" fn OnInvalid<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            result__: *mut windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardTranslator_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            match IKeyboardTranslator_Impl::OnInvalid(
                this,
                windows_core::from_raw_borrowed(&handler),
            ) {
                Ok(ok__) => {
                    result__.write(core::mem::transmute_copy(&ok__));
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn RemoveOnInvalid<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            token: windows::Foundation::EventRegistrationToken,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardTranslator_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            IKeyboardTranslator_Impl::RemoveOnInvalid(this, core::mem::transmute(&token)).into()
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IKeyboardTranslator, OFFSET>(),
            TranslateAndForward: TranslateAndForward::<Identity, OFFSET>,
            CheckLayoutAndUpdate: CheckLayoutAndUpdate::<Identity, OFFSET>,
            OnTranslated: OnTranslated::<Identity, OFFSET>,
            RemoveOnTranslated: RemoveOnTranslated::<Identity, OFFSET>,
            OnInvalid: OnInvalid::<Identity, OFFSET>,
            RemoveOnInvalid: RemoveOnInvalid::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IKeyboardTranslator as windows_core::Interface>::IID
    }
}
pub trait IKeyboardTranslatorFactory_Impl: Sized {
    fn CreateInstance(
        &self,
        definition: Option<&SequenceDefinition>,
    ) -> windows_core::Result<KeyboardTranslator>;
}
impl windows_core::RuntimeName for IKeyboardTranslatorFactory {
    const NAME: &'static str = "LibSimbolMudah.IKeyboardTranslatorFactory";
}
impl IKeyboardTranslatorFactory_Vtbl {
    pub const fn new<Identity: windows_core::IUnknownImpl, const OFFSET: isize>(
    ) -> IKeyboardTranslatorFactory_Vtbl
    where
        Identity: IKeyboardTranslatorFactory_Impl,
    {
        unsafe extern "system" fn CreateInstance<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            definition: *mut core::ffi::c_void,
            result__: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT
        where
            Identity: IKeyboardTranslatorFactory_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            match IKeyboardTranslatorFactory_Impl::CreateInstance(
                this,
                windows_core::from_raw_borrowed(&definition),
            ) {
                Ok(ok__) => {
                    result__.write(core::mem::transmute_copy(&ok__));
                    core::mem::forget(ok__);
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<
                Identity,
                IKeyboardTranslatorFactory,
                OFFSET,
            >(),
            CreateInstance: CreateInstance::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IKeyboardTranslatorFactory as windows_core::Interface>::IID
    }
}
pub trait ISequenceDefinition_Impl: Sized {
    fn Build(
        &self,
        keysymdef: &windows_core::HSTRING,
        composedef: &windows_core::HSTRING,
    ) -> windows_core::Result<()>;
}
impl windows_core::RuntimeName for ISequenceDefinition {
    const NAME: &'static str = "LibSimbolMudah.ISequenceDefinition";
}
impl ISequenceDefinition_Vtbl {
    pub const fn new<Identity: windows_core::IUnknownImpl, const OFFSET: isize>(
    ) -> ISequenceDefinition_Vtbl
    where
        Identity: ISequenceDefinition_Impl,
    {
        unsafe extern "system" fn Build<Identity: windows_core::IUnknownImpl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            keysymdef: core::mem::MaybeUninit<windows_core::HSTRING>,
            composedef: core::mem::MaybeUninit<windows_core::HSTRING>,
        ) -> windows_core::HRESULT
        where
            Identity: ISequenceDefinition_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            ISequenceDefinition_Impl::Build(
                this,
                core::mem::transmute(&keysymdef),
                core::mem::transmute(&composedef),
            )
            .into()
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, ISequenceDefinition, OFFSET>(),
            Build: Build::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ISequenceDefinition as windows_core::Interface>::IID
    }
}
pub trait ISequenceSearcher_Impl: Sized {
    fn Search(
        &self,
        keyword: &windows_core::HSTRING,
    ) -> windows_core::Result<windows::Foundation::Collections::IVectorView<SequenceDescription>>;
}
impl windows_core::RuntimeName for ISequenceSearcher {
    const NAME: &'static str = "LibSimbolMudah.ISequenceSearcher";
}
impl ISequenceSearcher_Vtbl {
    pub const fn new<Identity: windows_core::IUnknownImpl, const OFFSET: isize>(
    ) -> ISequenceSearcher_Vtbl
    where
        Identity: ISequenceSearcher_Impl,
    {
        unsafe extern "system" fn Search<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            keyword: core::mem::MaybeUninit<windows_core::HSTRING>,
            result__: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT
        where
            Identity: ISequenceSearcher_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            match ISequenceSearcher_Impl::Search(this, core::mem::transmute(&keyword)) {
                Ok(ok__) => {
                    result__.write(core::mem::transmute_copy(&ok__));
                    core::mem::forget(ok__);
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, ISequenceSearcher, OFFSET>(),
            Search: Search::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ISequenceSearcher as windows_core::Interface>::IID
    }
}
pub trait ISequenceSearcherFactory_Impl: Sized {
    fn CreateInstance(
        &self,
        definition: Option<&SequenceDefinition>,
    ) -> windows_core::Result<SequenceSearcher>;
}
impl windows_core::RuntimeName for ISequenceSearcherFactory {
    const NAME: &'static str = "LibSimbolMudah.ISequenceSearcherFactory";
}
impl ISequenceSearcherFactory_Vtbl {
    pub const fn new<Identity: windows_core::IUnknownImpl, const OFFSET: isize>(
    ) -> ISequenceSearcherFactory_Vtbl
    where
        Identity: ISequenceSearcherFactory_Impl,
    {
        unsafe extern "system" fn CreateInstance<
            Identity: windows_core::IUnknownImpl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            definition: *mut core::ffi::c_void,
            result__: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT
        where
            Identity: ISequenceSearcherFactory_Impl,
        {
            let this: &Identity = &*((this as *const *const ()).offset(OFFSET) as *const Identity);
            match ISequenceSearcherFactory_Impl::CreateInstance(
                this,
                windows_core::from_raw_borrowed(&definition),
            ) {
                Ok(ok__) => {
                    result__.write(core::mem::transmute_copy(&ok__));
                    core::mem::forget(ok__);
                    windows_core::HRESULT(0)
                }
                Err(err) => err.into(),
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<
                Identity,
                ISequenceSearcherFactory,
                OFFSET,
            >(),
            CreateInstance: CreateInstance::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ISequenceSearcherFactory as windows_core::Interface>::IID
    }
}
