mod cldr;
mod compose_reader;
mod keysym_reader;
mod mapped_string;

use core::str;
use std::{collections::HashMap, rc::Rc};

use crate::{bindings, utils::functions::fail};
use cldr::{load_annotation_file, AnnotationPair, SupportedLocale};
use compose_reader::ComposeDef;
use fst::{automaton::Str, Automaton, IntoStreamer, Map, MapBuilder, Streamer};
use keysym_reader::KeySymDef;
use mapped_string::MappedString;
use smol_str::{SmolStr, ToSmolStr};
use windows::{
    core::{implement, Error, IInspectable, Result, HSTRING, PSTR},
    Foundation::Collections::IVectorView,
    Globalization::Language,
    System::UserProfile::GlobalizationPreferences,
    Win32::{
        Foundation::E_NOTIMPL,
        Globalization::{u_charName, UErrorCode, U_EXTENDED_CHAR_NAME},
        System::WinRT::{IActivationFactory, IActivationFactory_Impl},
    },
};

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
    prefix_map: Map<Vec<u8>>,
    value_to_string: HashMap<u64, MappedString>,
    char_to_name: HashMap<SmolStr, Box<str>>,
    string_to_sequence: HashMap<String, String>,
    annotations: HashMap<SupportedLocale, Box<[AnnotationPair]>>,
}

impl SequenceDefinition {
    pub(crate) fn translate_sequence(
        &self,
        sequence: &str,
    ) -> std::result::Result<String, SequenceDefinitionError> {
        self.prefix_map.get(sequence.as_bytes()).map_or_else(
            || {
                Err(if self.potential_prefix(sequence, 1).is_empty() {
                    SequenceDefinitionError::ValueNotFound
                } else {
                    SequenceDefinitionError::Incomplete
                })
            },
            |value| {
                Ok(self
                    .value_to_string
                    .get(&value)
                    .expect("value previously mapped")
                    .to_string())
            },
        )
    }

    fn potential_prefix(&self, sequence: &str, limit: usize) -> Vec<bindings::SequenceDescription> {
        let mut stream = self
            .prefix_map
            .search(Str::new(sequence).starts_with())
            .into_stream();
        let mut result = Vec::with_capacity(limit);

        for _ in 0..limit {
            if let Some(element) = stream.next() {
                let (seq, value) = element;
                let mapped_value = self
                    .value_to_string
                    .get(&value)
                    .expect("value previously mapped");

                result.push(bindings::SequenceDescription {
                    sequence: unsafe { str::from_utf8_unchecked(seq) }.into(),
                    result: mapped_value.to_string().into(),
                    description: match mapped_value {
                        MappedString::Basic(c) => {
                            self.char_to_name.get(c).unwrap().to_string().into()
                        }
                        MappedString::Extra(s) => s.to_string().into(),
                    },
                });
            } else {
                return result;
            }
        }

        result
    }

    fn filter_sequence(
        &self,
        tokens: Vec<String>,
        limit: usize,
        languages: &[SupportedLocale],
    ) -> Vec<bindings::SequenceDescription> {
        let mut result_map = HashMap::new();

        // prioritize exact character match
        for lang in languages {
            let annotations = self.annotations.get(lang).unwrap();
            for pair in annotations
                .iter()
                .filter(|pair| tokens.iter().any(|t| pair.char.contains(&t.to_lowercase())))
            {
                if !result_map.contains_key(&pair.char.to_string()) {
                    result_map.insert(
                        pair.char.to_string(),
                        self.char_to_name
                            .get(&pair.char.to_smolstr())
                            .expect("already indexed")
                            .to_string(),
                    );
                }
            }

            if result_map.len() >= limit {
                return self.process_map(&result_map);
            }
        }

        for token in tokens.iter() {
            if !result_map.contains_key(token) {
                let token = token.to_smolstr();
                if let Some(name) = self.char_to_name.get(&token) {
                    result_map.insert(token.to_string(), name.to_string());
                }
            }
        }

        if result_map.len() >= limit {
            return self.process_map(&result_map);
        }

        // search in descriptions
        for lang in languages {
            let annotations = self.annotations.get(lang).unwrap();
            for pair in annotations
                .iter()
                .filter(|pair| tokens.iter().all(|t| pair.desc.contains(&t.to_lowercase())))
            {
                if !result_map.contains_key(&pair.char.to_string()) {
                    result_map.insert(
                        pair.char.to_string(),
                        self.char_to_name
                            .get(&pair.char.to_smolstr())
                            .expect("already indexed")
                            .to_string(),
                    );
                }
            }

            if result_map.len() >= limit {
                return self.process_map(&result_map);
            }
        }

        for (c, n) in self
            .char_to_name
            .iter()
            .filter(|(_, n)| tokens.iter().all(|t| n.contains(&t.to_uppercase())))
        {
            let c = c.to_string();
            if !result_map.contains_key(&c) {
                result_map.insert(c, n.to_string());
            }
        }

        self.process_map(&result_map)
    }

    fn process_map(&self, map: &HashMap<String, String>) -> Vec<bindings::SequenceDescription> {
        let mut result = Vec::with_capacity(map.len());
        for (char, desc) in map {
            let given_sequence = if let Some(sequence) = self.string_to_sequence.get(char) {
                sequence.into()
            } else {
                HSTRING::from("")
            };

            result.push(bindings::SequenceDescription {
                sequence: given_sequence,
                result: char.into(),
                description: desc.into(),
            });
        }

        return result;
    }

    fn tokenize(&self, keyword: &HSTRING) -> Vec<String> {
        keyword
            .to_string()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
}

impl bindings::ISequenceDefinition_Impl for SequenceDefinition_Impl {
    fn PotentialPrefix(
        &self,
        sequence: &HSTRING,
        limit: u32,
    ) -> Result<IVectorView<bindings::SequenceDescription>> {
        self.potential_prefix(&sequence.to_string(), limit as usize)
            .try_into()
    }

    fn Search(
        &self,
        sequence: &HSTRING,
        limit: u32,
    ) -> Result<IVectorView<bindings::SequenceDescription>> {
        let user_langs = get_user_langs()?;
        self.filter_sequence(self.tokenize(sequence), limit as usize, &user_langs)
            .try_into()
    }
}

#[implement(IActivationFactory, bindings::ISequenceDefinitionFactory)]
pub(super) struct SequenceDefinitionFactory;

impl IActivationFactory_Impl for SequenceDefinitionFactory_Impl {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Err(E_NOTIMPL.into())
    }
}

impl bindings::ISequenceDefinitionFactory_Impl for SequenceDefinitionFactory_Impl {
    fn CreateInstance(
        &self,
        keysymdef: &HSTRING,
        composedef: &HSTRING,
    ) -> Result<bindings::SequenceDefinition> {
        let keysymdef = KeySymDef::new(&keysymdef.to_string())?;
        let composedef = ComposeDef::build(&keysymdef, &composedef.to_string())?;

        let mut annotations = HashMap::new();
        let mut char_to_name: HashMap<SmolStr, Box<str>> = HashMap::new();
        let languages = get_user_langs()?;
        for locale in languages.iter() {
            let mut result_vec = Vec::new();
            for variant in ["annotations", "annotationsDerived"] {
                for a in load_annotation_file(&format!(
                    "ms-appx:///Assets/Annotations/{locale}-{variant}.xml.br"
                ))? {
                    if a.r#type.is_some() {
                        if locale == languages.first().expect("there is at least one language") {
                            char_to_name.insert(a.cp.into(), Box::from(a.text));
                        }
                    } else {
                        let main_char: Rc<str> = Rc::from(a.cp);
                        let words = a
                            .text
                            .split_terminator("|")
                            .map(|s| s.trim())
                            .collect::<Vec<_>>();
                        for word in words {
                            result_vec.push(AnnotationPair {
                                char: main_char.clone(),
                                desc: Box::from(word),
                            });
                        }
                    }
                }
            }

            annotations.insert(*locale, result_vec.into_boxed_slice());
        }

        let mut build = MapBuilder::memory();
        let mut value_to_string = HashMap::new();
        let mut string_to_sequence = HashMap::new();
        let mut basic_index = 0;
        let mut extra_index = u32::MAX.into();

        for (key, value) in composedef {
            match value {
                MappedString::Basic(e) => {
                    basic_index += 1;
                    value_to_string.insert(basic_index, MappedString::Basic(e.clone()));
                    string_to_sequence.insert(e.to_string(), key.clone());
                    build.insert(key.clone(), basic_index).map_err(fail)?;
                    char_to_name.insert(
                        e.clone(),
                        char_to_unicode_name(e.chars().next().expect("string not empty"))?,
                    );
                }
                MappedString::Extra(_) => {
                    extra_index += 1;
                    value_to_string.insert(extra_index, value.clone());
                    string_to_sequence.insert(value.to_string(), key.clone());
                    build.insert(key.clone(), extra_index).map_err(fail)?;
                }
            }
        }

        let instance: bindings::SequenceDefinition = SequenceDefinition {
            prefix_map: build.into_map(),
            value_to_string,
            char_to_name,
            string_to_sequence,
            annotations,
        }
        .into();

        Ok(instance)
    }
}

fn get_user_langs() -> Result<Box<[SupportedLocale]>> {
    let user_langs = GlobalizationPreferences::Languages()?;
    let mut valid_langs = Vec::new();
    for lang in user_langs {
        valid_langs.push(Language::CreateLanguage(&lang)?.try_into()?);
    }
    Ok(valid_langs.into_boxed_slice())
}

fn char_to_unicode_name(value: char) -> Result<Box<str>> {
    let mut buffer = [0; 88];
    let pstr = PSTR::from_raw(buffer.as_mut_ptr());
    let mut errcode = UErrorCode::default();
    let len = unsafe { u_charName(value as i32, U_EXTENDED_CHAR_NAME, pstr, 88, &mut errcode) };

    assert!(len.is_positive());
    let name = unsafe { pstr.to_string() }.map_err(fail)?;

    assert_eq!(len as usize, name.len());
    Ok(name.into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bindings::ISequenceDefinitionFactory_Impl;
    use windows_core::{Interface, Result};

    const KEYSYMDEF: &str = "tests/keysymdef.txt";
    const COMPOSEDEF: &str = "tests/Compose.pre";

    #[test]
    fn test_check_languages() -> Result<()> {
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()?
            .ActivateInstance()?;

        let _seqdef = seqdef.cast_object_ref::<SequenceDefinition>()?;

        // print BCP-47 language tag
        let user_langs = GlobalizationPreferences::Languages()?;
        for lang in user_langs.into_iter() {
            println!("BCP-47: {:?}", lang);
        }

        Ok(())
    }

    #[test]
    fn test_translate_incomplete_sequence() -> Result<()> {
        // Create and build the SequenceDefinition
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()?
            .CreateInstance(&KEYSYMDEF.into(), &COMPOSEDEF.into())?;

        // Cast SequenceDefinition to its object
        let seqdef_ref = seqdef.cast_object_ref::<SequenceDefinition>()?;

        // Attempt to translate an incomplete sequence
        let result = seqdef_ref.translate_sequence("f");
        assert!(matches!(result, Err(SequenceDefinitionError::Incomplete)));
        Ok(())
    }

    #[test]
    fn test_translate_value_not_found() -> Result<()> {
        // Create and build the SequenceDefinition
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()?
            .CreateInstance(&KEYSYMDEF.into(), &COMPOSEDEF.into())?;

        // Cast SequenceDefinition to its object
        let seqdef = seqdef.cast_object_ref::<SequenceDefinition>()?;

        // Attempt to translate a nonexistent sequence
        let result = seqdef.translate_sequence("nonexistent");
        assert!(matches!(
            result,
            Err(SequenceDefinitionError::ValueNotFound)
        ));
        Ok(())
    }

    #[test]
    fn test_translate_valid_sequence() -> Result<()> {
        // Create and build the SequenceDefinition
        let factory: IActivationFactory = SequenceDefinitionFactory.into();
        let seqdef = factory
            .cast_object_ref::<SequenceDefinitionFactory>()?
            .CreateInstance(&KEYSYMDEF.into(), &COMPOSEDEF.into())?;

        // Cast SequenceDefinition to its object
        let seqdef = seqdef.cast_object_ref::<SequenceDefinition>()?;

        // Assuming "fl" is a valid sequence mapped to a basic MappedString for this test
        let result = seqdef.translate_sequence("fl");
        assert!(result.is_ok());
        let expected = "ﬂ"; // Expected result for the sequence "fl"
        assert_eq!(result.unwrap(), expected);
        Ok(())
    }
}
