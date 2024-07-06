mod internal;

use std::sync::{Arc, RwLock};

use crate::{
    bindings,
    delegate_storage::{get_token, DelegateStorage},
};
use internal::SequenceSearcherInternal;
use windows::{
    core::{implement, AgileReference, Error, IInspectable, Interface, Result, Weak, HSTRING},
    Foundation::{
        AsyncStatus, Collections::IVectorView, EventRegistrationToken, IAsyncAction,
        TypedEventHandler,
    },
    System::Threading::{ThreadPool, WorkItemHandler},
    Win32::{
        Foundation::{E_ABORT, E_POINTER},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
    },
};

type SS = bindings::SequenceSearcher;
type SD = bindings::SequenceDescription;
type D = TypedEventHandler<SS, IVectorView<SD>>;

#[implement(SS)]
struct SequenceSearcher {
    internal: Arc<RwLock<SequenceSearcherInternal>>,
    on_result: Arc<RwLock<DelegateStorage<SS, IVectorView<SD>>>>,
}

impl bindings::ISequenceSearcher_Impl for SequenceSearcher_Impl {
    fn Search(&self, _keyword: &HSTRING) -> Result<IAsyncAction> {
        let internal = self.internal.clone();
        let on_result = self.on_result.clone();

        ThreadPool::RunAsync(&WorkItemHandler::new(move |a| {
            if let Some(a) = a {
                if a.Status()? == AsyncStatus::Canceled {
                    return Err(Error::new(E_ABORT, "Operation canceled"));
                }

                let args = IVectorView::try_from(vec![Some(SD::new()?), Some(SD::new()?)])?;
                on_result.write().unwrap().invoke_all(
                    &internal.read().unwrap().parent.upgrade().unwrap(),
                    Some(&args),
                )
            } else {
                Err(Error::new(E_POINTER, "Null pointer"))
            }
        }))
    }

    fn OnSearchResult(&self, handler: Option<&D>) -> Result<EventRegistrationToken> {
        if let Some(handler) = handler {
            let token = get_token(handler.as_raw());
            self.on_result
                .write()
                .unwrap()
                .insert(token, AgileReference::new(handler)?);

            Ok(EventRegistrationToken { Value: token })
        } else {
            Err(Error::new(E_POINTER, "Null pointer"))
        }
    }

    fn RemoveOnSearchResult(&self, token: &EventRegistrationToken) -> Result<()> {
        Ok(self.on_result.write().unwrap().remove(token.Value))
    }
}

#[implement(IActivationFactory)]
struct SequenceSearcherFactory;

impl IActivationFactory_Impl for SequenceSearcherFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        let internal = SequenceSearcherInternal::new(Weak::new());
        let instance: SS = SequenceSearcher {
            internal: Arc::new(RwLock::new(internal)),
            on_result: Arc::new(RwLock::new(DelegateStorage::new())),
        }
        .into();

        instance
            .cast_object::<SequenceSearcher>()?
            .internal
            .try_write()
            .unwrap()
            .parent = instance.downgrade()?;

        Ok(instance.into())
    }
}
