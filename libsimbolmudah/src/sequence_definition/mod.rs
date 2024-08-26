mod cldr;
mod compose_reader;
mod internal;
mod keysym_reader;
mod mapped_string;

use std::sync::RwLock;

use crate::{bindings, utils::functions::fail};
use cldr::SupportedLocale;
use internal::SequenceDefinitionInternal;
use windows::{
    core::{implement, Error, IInspectable, Result, Weak, HSTRING},
    Foundation::Collections::IVectorView,
    Globalization::Language,
    System::UserProfile::GlobalizationPreferences,
    Win32::System::WinRT::{IActivationFactory, IActivationFactory_Impl},
};
use windows_core::Interface;

#[derive(Debug, PartialEq)]
pub(crate) enum SequenceDefinitionError {
    ValueNotFound,
    Incomplete,
    Failure(Error),
}

impl From<Error> for SequenceDefinitionError {
    fn from(error: Error) -> Self {
        Self::Failure(error)
    }
}

#[implement(bindings::SequenceDefinition)]
pub(crate) struct SequenceDefinition {
    internal: RwLock<SequenceDefinitionInternal>,
}

impl SequenceDefinition {
    pub(crate) fn translate_sequence(
        &self,
        sequence: &str,
    ) -> std::result::Result<String, SequenceDefinitionError> {
        self.internal
            .read()
            .map_err(fail)?
            .translate_sequence(sequence)
    }

    fn tokenize(&self, keyword: &HSTRING) -> Vec<String> {
        keyword
            .to_string()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    fn get_user_langs(&self) -> Result<Box<[SupportedLocale]>> {
        let user_langs = GlobalizationPreferences::Languages()?;
        let mut valid_langs = Vec::new();
        for lang in user_langs {
            valid_langs.push(Language::CreateLanguage(&lang)?.try_into()?);
        }
        Ok(valid_langs.into_boxed_slice())
    }
}

impl bindings::ISequenceDefinition_Impl for SequenceDefinition_Impl {
    fn Build(&self, keysymdef: &HSTRING, composedef: &HSTRING) -> Result<()> {
        self.internal.write().map_err(fail)?.build(
            keysymdef.to_string().as_str(),
            composedef.to_string().as_str(),
            &self.get_user_langs()?,
        )
    }

    fn PotentialPrefix(
        &self,
        sequence: &HSTRING,
        limit: u32,
    ) -> Result<IVectorView<bindings::SequenceDescription>> {
        self.internal
            .read()
            .map_err(fail)?
            .potential_prefix(sequence.to_string().as_str(), limit as usize)
            .try_into()
    }

    fn Search(
        &self,
        sequence: &HSTRING,
        limit: u32,
    ) -> Result<IVectorView<bindings::SequenceDescription>> {
        let user_langs = self.get_user_langs()?;
        self.internal
            .read()
            .map_err(fail)?
            .filter_sequence(self.tokenize(sequence), limit as usize, &user_langs)
            .try_into()
    }
}

#[implement(IActivationFactory)]
pub(super) struct SequenceDefinitionFactory;

impl IActivationFactory_Impl for SequenceDefinitionFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        let internal = SequenceDefinitionInternal::new(Weak::new());
        let instance: bindings::SequenceDefinition = SequenceDefinition {
            internal: RwLock::new(internal),
        }
        .into();

        instance
            .cast_object::<SequenceDefinition>()?
            .internal
            .try_write()
            .map_err(fail)?
            .parent = instance.downgrade()?;

        Ok(instance.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_core::Interface;

    use crate::bindings::ISequenceDefinition_Impl;

    const KEYSYMDEF: &str = "tests/keysymdef.txt";
    const COMPOSEDEF: &str = "tests/Compose.pre";

    #[test]
    fn test_check_languages() -> Result<()> {
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()?
            .ActivateInstance()?;

        let seqdef = seqdef.cast_object_ref::<SequenceDefinition>()?;
        let _langs = seqdef.get_user_langs()?;

        // print BCP-47 language tag
        let user_langs = GlobalizationPreferences::Languages()?;
        for lang in user_langs.into_iter() {
            println!("BCP-47: {:?}", lang);
        }

        Ok(())
    }

    #[test]
    fn test_build_success() {
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()
            .unwrap()
            .ActivateInstance()
            .expect("SequenceDefinition should be created");

        seqdef
            .cast_object_ref::<SequenceDefinition>()
            .expect("SequenceDefinition should be casted")
            .Build(&KEYSYMDEF.into(), &COMPOSEDEF.into())
            .expect("SequenceDefinition should be built");
    }

    #[test]
    fn test_translate_incomplete_sequence() {
        // Create and build the SequenceDefinition
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()
            .unwrap()
            .ActivateInstance()
            .expect("SequenceDefinition should be created");
        seqdef
            .cast_object_ref::<SequenceDefinition>()
            .expect("SequenceDefinition should be casted")
            .Build(&KEYSYMDEF.into(), &COMPOSEDEF.into())
            .expect("SequenceDefinition should be built");

        // Cast SequenceDefinition to its object
        let seqdef_ref = seqdef
            .cast_object_ref::<SequenceDefinition>()
            .expect("SequenceDefinition should be casted");

        // Attempt to translate an incomplete sequence
        let result = seqdef_ref.translate_sequence("f");
        assert!(matches!(result, Err(SequenceDefinitionError::Incomplete)));
    }

    #[test]
    fn test_translate_value_not_found() {
        // Create and build the SequenceDefinition
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()
            .unwrap()
            .ActivateInstance()
            .expect("SequenceDefinition should be created");
        seqdef
            .cast_object_ref::<SequenceDefinition>()
            .expect("SequenceDefinition should be casted")
            .Build(&KEYSYMDEF.into(), &COMPOSEDEF.into())
            .expect("SequenceDefinition should be built");

        // Cast SequenceDefinition to its object
        let seqdef = seqdef
            .cast_object_ref::<SequenceDefinition>()
            .expect("SequenceDefinition should be casted");

        // Attempt to translate a nonexistent sequence
        let result = seqdef.translate_sequence("nonexistent");
        assert!(matches!(
            result,
            Err(SequenceDefinitionError::ValueNotFound)
        ));
    }

    #[test]
    fn test_translate_valid_sequence() {
        // Create and build the SequenceDefinition
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()
            .unwrap()
            .ActivateInstance()
            .expect("SequenceDefinition should be created");
        seqdef
            .cast_object_ref::<SequenceDefinition>()
            .expect("SequenceDefinition should be casted")
            .Build(&KEYSYMDEF.into(), &COMPOSEDEF.into())
            .expect("SequenceDefinition should be built");

        // Cast SequenceDefinition to its object
        let seqdef = seqdef
            .cast_object_ref::<SequenceDefinition>()
            .expect("SequenceDefinition should be casted");

        // Assuming "fl" is a valid sequence mapped to a basic MappedString for this test
        let result = seqdef.translate_sequence("fl");
        assert!(result.is_ok());
        let expected = "ﬂ"; // Expected result for the sequence "fl"
        assert_eq!(result.unwrap(), expected);
    }
}
