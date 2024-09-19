mod cldr;
mod compose_reader;
mod keysym_reader;
mod mapped_string;

use core::str;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{RwLock, RwLockReadGuard},
};

use crate::{bindings, utils::functions::fail};
use cldr::{load_annotation_file, AnnotationPair, SupportedLocale};
use compose_reader::ComposeDef;
use fst::{automaton::Str, Automaton, IntoStreamer, Map, MapBuilder, Streamer};
use keysym_reader::KeySymDef;
use mapped_string::MappedString;
use windows::{
    core::{h, implement, Error, IInspectable, HSTRING, PSTR},
    Foundation::Collections::IVectorView,
    Globalization::Language,
    Storage::StorageFile,
    System::UserProfile::GlobalizationPreferences,
    Win32::{
        Foundation::{ERROR_NO_UNICODE_TRANSLATION, E_INVALIDARG},
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

impl Into<Error> for SequenceDefinitionError {
    fn into(self) -> Error {
        match self {
            Self::ValueNotFound => ERROR_NO_UNICODE_TRANSLATION.into(),
            Self::Incomplete => Error::new(E_INVALIDARG, "Incomplete sequence"),
            Self::Failure(e) => e,
        }
    }
}

#[implement(bindings::SequenceDefinition)]
#[derive(Default)]
pub(crate) struct SequenceDefinition {
    prefix_map: RwLock<Map<Vec<u8>>>,
    value_to_string: RwLock<HashMap<u64, MappedString>>,
    char_to_name: RwLock<HashMap<String, Box<str>>>,
    string_to_sequence: RwLock<HashMap<String, String>>,
    annotations: RwLock<HashMap<SupportedLocale, Box<[AnnotationPair]>>>,
}

impl SequenceDefinition {
    pub(crate) fn translate_sequence(
        &self,
        sequence: &str,
    ) -> Result<String, SequenceDefinitionError> {
        read_lock(&self.prefix_map)?
            .get(sequence.as_bytes())
            .map_or_else(
                || {
                    Err(if self.potential_prefix(sequence, 1)?.is_empty() {
                        SequenceDefinitionError::ValueNotFound
                    } else {
                        SequenceDefinitionError::Incomplete
                    })
                },
                |value| {
                    Ok(read_lock(&self.value_to_string)?
                        .get(&value)
                        .expect("value previously mapped")
                        .to_string())
                },
            )
    }

    fn potential_prefix(
        &self,
        sequence: &str,
        limit: usize,
    ) -> Result<Vec<bindings::SequenceDescription>, SequenceDefinitionError> {
        let prefix_map = read_lock(&self.prefix_map)?;
        let mut stream = prefix_map
            .search(Str::new(sequence).starts_with())
            .into_stream();
        let mut result = Vec::with_capacity(limit);

        for _ in 0..limit {
            if let Some(element) = stream.next() {
                let (seq, value) = element;
                let mapped_value = read_lock(&self.value_to_string)?
                    .get(&value)
                    .expect("value previously mapped")
                    .clone();

                result.push(bindings::SequenceDescription {
                    sequence: unsafe { str::from_utf8_unchecked(seq) }.into(),
                    result: mapped_value.to_string().into(),
                    description: match mapped_value {
                        MappedString::Basic(c) => read_lock(&self.char_to_name)?
                            .get(&c.to_string())
                            .unwrap()
                            .to_string()
                            .into(),
                        MappedString::Extra(s) => s.to_string().into(),
                    },
                });
            } else {
                return Ok(result);
            }
        }

        Ok(result)
    }

    fn filter_sequence(
        &self,
        tokens: Vec<String>,
        limit: usize,
        languages: &[SupportedLocale],
    ) -> Result<Vec<bindings::SequenceDescription>, SequenceDefinitionError> {
        let mut result_map = HashMap::new();

        // prioritize exact character match
        for lang in languages {
            let annotation_map = read_lock(&self.annotations)?;
            for pair in annotation_map
                .get(lang)
                .expect("language supported")
                .iter()
                .filter(|pair| tokens.iter().any(|t| pair.char.contains(&t.to_lowercase())))
            {
                if !result_map.contains_key(&pair.char.to_string()) {
                    result_map.insert(
                        pair.char.to_string(),
                        read_lock(&self.char_to_name)?
                            .get(&pair.char.to_string())
                            .expect("already indexed")
                            .to_string(),
                    );
                }

                if result_map.len() >= limit {
                    return self.process_map(&result_map);
                }
            }
        }

        for token in tokens.iter() {
            if !result_map.contains_key(token) {
                if let Some(name) = read_lock(&self.char_to_name)?.get(&token.to_string()) {
                    result_map.insert(token.to_string(), name.to_string());
                }
            }

            if result_map.len() >= limit {
                return self.process_map(&result_map);
            }
        }

        // search in descriptions
        for lang in languages {
            let annotation_map = read_lock(&self.annotations)?;
            for pair in annotation_map
                .get(lang)
                .expect("language supported")
                .iter()
                .filter(|pair| tokens.iter().all(|t| pair.desc.contains(&t.to_lowercase())))
            {
                if !result_map.contains_key(&pair.char.to_string()) {
                    result_map.insert(
                        pair.char.to_string(),
                        read_lock(&self.char_to_name)?
                            .get(&pair.char.to_string())
                            .expect("already indexed")
                            .to_string(),
                    );
                }

                if result_map.len() >= limit {
                    return self.process_map(&result_map);
                }
            }
        }

        for (c, n) in read_lock(&self.char_to_name)?
            .iter()
            .filter(|(_, n)| tokens.iter().all(|t| n.contains(&t.to_uppercase())))
        {
            let c = c.to_string();
            if !result_map.contains_key(&c) {
                result_map.insert(c, n.to_string());
            }

            if result_map.len() >= limit {
                return self.process_map(&result_map);
            }
        }

        self.process_map(&result_map)
    }

    fn process_map(
        &self,
        map: &HashMap<String, String>,
    ) -> Result<Vec<bindings::SequenceDescription>, SequenceDefinitionError> {
        let mut result = Vec::with_capacity(map.len());
        for (char, desc) in map {
            let given_sequence =
                if let Some(sequence) = read_lock(&self.string_to_sequence)?.get(char) {
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

        Ok(result)
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
    fn Rebuild(
        &self,
        keysymdef: &HSTRING,
        composedef: &HSTRING,
        annotations: &HSTRING,
    ) -> windows_core::Result<()> {
        let keysymdef = KeySymDef::new(&keysymdef.to_string())?;
        let composedef = ComposeDef::build(&keysymdef, &composedef.to_string())?;

        let mut annotation_map = HashMap::new();
        let mut char_to_name: HashMap<String, Box<str>> = HashMap::new();
        let languages = get_user_langs()?;
        let prefetch = languages
            .iter()
            .map(|locale| {
                (
                    locale,
                    ["annotations", "annotationsDerived"].map(move |variant| {
                        StorageFile::GetFileFromPathAsync(
                            &format!("{}\\{locale}-{variant}.xml.br", annotations.to_string())
                                .into(),
                        )
                    }),
                )
            })
            .collect::<Vec<_>>();

        for (locale, files) in prefetch {
            let mut result_vec = Vec::new();
            for variant in files {
                for a in load_annotation_file(&variant?.get()?.Path()?)? {
                    if a.r#type.is_some() {
                        if !char_to_name.contains_key(&a.cp) {
                            char_to_name.insert(a.cp, Box::from(a.text));
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

            annotation_map.insert(*locale, result_vec.into_boxed_slice());
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
                        e.to_string(),
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

        *self.prefix_map.write().map_err(fail)? = build.into_map();
        *self.value_to_string.write().map_err(fail)? = value_to_string;
        *self.char_to_name.write().map_err(fail)? = char_to_name;
        *self.string_to_sequence.write().map_err(fail)? = string_to_sequence;
        *self.annotations.write().map_err(fail)? = annotation_map;

        Ok(())
    }

    fn PotentialPrefix(
        &self,
        sequence: &HSTRING,
        limit: u32,
    ) -> windows_core::Result<IVectorView<bindings::SequenceDescription>> {
        self.potential_prefix(&sequence.to_string(), limit as usize)
            .map_err(|e| Into::<Error>::into(e))?
            .try_into()
    }

    fn Search(
        &self,
        sequence: &HSTRING,
        limit: u32,
    ) -> windows_core::Result<IVectorView<bindings::SequenceDescription>> {
        let user_langs = get_user_langs()?;
        self.filter_sequence(self.tokenize(sequence), limit as usize, &user_langs)
            .map_err(|e| Into::<Error>::into(e))?
            .try_into()
    }

    fn GetLocalizedName(
        &self,
        codepoint: u32,
    ) -> windows_core::Result<bindings::SequenceDescription> {
        let valid_char = char::from_u32(codepoint).ok_or_else(|| ERROR_NO_UNICODE_TRANSLATION)?;
        let valid_string = valid_char.to_string();
        let description = if let Some(mapped) = read_lock(&self.char_to_name)
            .map_err(|e| Into::<Error>::into(e))?
            .get(&valid_string)
        {
            mapped.to_string()
        } else {
            char_to_unicode_name(valid_char)?.to_string()
        }
        .into();

        Ok(bindings::SequenceDescription {
            sequence: h!("").to_owned(),
            result: valid_string.into(),
            description,
        })
    }
}

#[implement(IActivationFactory)]
pub(super) struct SequenceDefinitionFactory;

impl IActivationFactory_Impl for SequenceDefinitionFactory_Impl {
    fn ActivateInstance(&self) -> windows_core::Result<IInspectable> {
        Ok(SequenceDefinition::default().into())
    }
}

fn get_user_langs() -> windows_core::Result<Box<[SupportedLocale]>> {
    let user_langs = GlobalizationPreferences::Languages()?;
    let mut valid_langs = Vec::new();
    for lang in user_langs {
        valid_langs.push(Language::CreateLanguage(&lang)?.try_into()?);
    }
    Ok(valid_langs.into_boxed_slice())
}

fn char_to_unicode_name(value: char) -> windows_core::Result<Box<str>> {
    let mut buffer = [0; 88];
    let pstr = PSTR::from_raw(buffer.as_mut_ptr());
    let mut errcode = UErrorCode::default();
    let len = unsafe { u_charName(value as i32, U_EXTENDED_CHAR_NAME, pstr, 88, &mut errcode) };

    assert!(len.is_positive());
    let name = unsafe { pstr.to_string() }.map_err(fail)?;

    assert_eq!(len as usize, name.len());
    Ok(name.into_boxed_str())
}

fn read_lock<T>(
    lock: &RwLock<T>,
) -> std::result::Result<RwLockReadGuard<'_, T>, SequenceDefinitionError> {
    lock.read().map_err(|e| fail(e).into())
}

#[cfg(test)]
mod tests {
    use std::str;

    use super::*;
    use windows_core::{ComObjectInner, Interface, Result};

    const KEYSYMDEF: &str = "x11-defs/keysymdef.h.br";
    const COMPOSEDEF: &str = "x11-defs/Compose.pre.br";
    const ANNOTATIONS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\cldr");

    #[test]
    fn test_check_languages() -> Result<()> {
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
        let seqdef = SequenceDefinitionFactory
            .into_object()
            .ActivateInstance()?
            .cast::<bindings::SequenceDefinition>()?;

        seqdef.Rebuild(&KEYSYMDEF.into(), &COMPOSEDEF.into(), &ANNOTATIONS.into())?;

        // Attempt to translate an incomplete sequence
        let result = seqdef
            .cast_object_ref::<SequenceDefinition>()?
            .translate_sequence("f");
        assert!(matches!(result, Err(SequenceDefinitionError::Incomplete)));
        Ok(())
    }

    #[test]
    fn test_translate_value_not_found() -> Result<()> {
        // Create and build the SequenceDefinition
        let seqdef = SequenceDefinitionFactory
            .into_object()
            .ActivateInstance()?
            .cast::<bindings::SequenceDefinition>()?;

        seqdef.Rebuild(&KEYSYMDEF.into(), &COMPOSEDEF.into(), &ANNOTATIONS.into())?;

        // Attempt to translate a nonexistent sequence
        let result = seqdef
            .cast_object_ref::<SequenceDefinition>()?
            .translate_sequence("nonexistent");
        assert!(matches!(
            result,
            Err(SequenceDefinitionError::ValueNotFound)
        ));
        Ok(())
    }

    #[test]
    fn test_translate_valid_sequence() -> Result<()> {
        // Create and build the SequenceDefinition
        let seqdef = SequenceDefinitionFactory
            .into_object()
            .ActivateInstance()?
            .cast::<bindings::SequenceDefinition>()?;

        seqdef.Rebuild(&KEYSYMDEF.into(), &COMPOSEDEF.into(), &ANNOTATIONS.into())?;

        // Assuming "fl" is a valid sequence mapped to a basic MappedString for this test
        let result = seqdef
            .cast_object_ref::<SequenceDefinition>()?
            .translate_sequence("fl");
        assert!(result.is_ok());
        let expected = "ﬂ"; // Expected result for the sequence "fl"
        assert_eq!(result.unwrap(), expected);
        Ok(())
    }

    #[test]
    fn test_unicode() -> Result<()> {
        let name: Box<str> = "#⃣".to_string().into();
        let reference: String = "#⃣".to_string();
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert(reference, "surprise".into());
        assert_eq!(map.get(&name.to_string()).unwrap(), "surprise");

        Ok(())
    }
}
