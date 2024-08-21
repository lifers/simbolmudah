use std::{collections::HashMap, fs::File, io::Read};

use fst::{automaton::Str, Automaton, IntoStreamer, Map, MapBuilder, Streamer};
use libsimbolmudah_cldr::{AnnotationPair, SupportedLocale};
use rkyv::{Deserialize, Infallible};
use windows::{
    core::{Result, Weak, HSTRING, PSTR},
    Foundation::Uri,
    Storage::StorageFile,
    Win32::Globalization::{u_charName, UErrorCode, U_EXTENDED_CHAR_NAME},
};

use crate::{bindings, utils::functions::fail};

use super::{
    compose_reader::ComposeDef, keysym_reader::KeySymDef, mapped_string::MappedString,
    SequenceDefinitionError,
};

pub(super) struct SequenceDefinitionInternal {
    prefix_map: Map<Vec<u8>>,
    index_map: HashMap<String, MappedString>,
    value_to_string: HashMap<u64, MappedString>,
    string_to_value: HashMap<MappedString, u64>,
    char_to_name: HashMap<char, String>,
    string_to_sequence: HashMap<String, String>,
    annotations: HashMap<SupportedLocale, Box<[AnnotationPair]>>,
    pub(super) parent: Weak<bindings::SequenceDefinition>,
}

impl SequenceDefinitionInternal {
    pub(super) fn new(parent: Weak<bindings::SequenceDefinition>) -> Self {
        Self {
            prefix_map: Map::default(),
            index_map: HashMap::new(),
            value_to_string: HashMap::new(),
            string_to_value: HashMap::new(),
            char_to_name: HashMap::new(),
            string_to_sequence: HashMap::new(),
            annotations: HashMap::new(),
            parent,
        }
    }

    fn load_cldr_annotations(&mut self) -> Result<()> {
        let mut result = HashMap::new();
        for locale in [
            SupportedLocale::en,
            SupportedLocale::id,
            SupportedLocale::fr,
            SupportedLocale::jv,
        ] {
            let path =
                StorageFile::GetFileFromApplicationUriAsync(&Uri::CreateUri(&HSTRING::from(
                    format!("ms-appx:///Assets/Annotations/annotations-{}.rkyv", locale),
                ))?)?
                .get()?
                .Path()?
                .to_string();
            let mut file = File::open(&path).expect("file opened successfully");
            let mut buf = Vec::new();
            let _bytes = file.read_to_end(&mut buf).expect("file read successfully");
            let archived = unsafe { rkyv::archived_root::<Box<[AnnotationPair]>>(&buf) };
            let annotations = archived
                .deserialize(&mut Infallible)
                .expect("bytes deserialized successfully");
            result.insert(locale, annotations);
        }

        self.annotations = result;
        Ok(())
    }

    pub(super) fn build(&mut self, keysymdef: &str, composedef: &str, cldrdir: &str) -> Result<()> {
        let keysymdef = KeySymDef::new(keysymdef, self)?;
        let composedef = ComposeDef::build(&keysymdef, composedef, self)?;
        self.load_cldr_annotations()?;
        let mut build = MapBuilder::memory();
        self.index_map.clear();
        self.value_to_string.clear();
        self.string_to_value.clear();
        self.string_to_sequence.clear();
        let mut basic_index = 0;
        let mut extra_index = u32::MAX.into();

        for (key, value) in composedef {
            match value {
                MappedString::Basic(_) => {
                    basic_index += 1;
                    self.value_to_string.insert(basic_index, value.clone());
                    self.string_to_value.insert(value.clone(), basic_index);
                    self.string_to_sequence
                        .insert(value.to_string(), key.clone());
                    build.insert(key.clone(), basic_index).map_err(fail)?;
                }
                MappedString::Extra(_) => {
                    extra_index += 1;
                    self.value_to_string.insert(extra_index, value.clone());
                    self.string_to_value.insert(value.clone(), extra_index);
                    self.string_to_sequence
                        .insert(value.to_string(), key.clone());
                    build.insert(key.clone(), extra_index).map_err(fail)?;
                }
            }

            self.index_map.insert(key, value);
        }

        self.prefix_map = build.into_map();
        Ok(())
    }

    pub(super) fn translate_sequence(
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

    pub(super) fn potential_prefix(
        &self,
        sequence: &str,
        limit: usize,
    ) -> Vec<bindings::SequenceDescription> {
        let mut stream = self
            .prefix_map
            .search(Str::new(sequence).starts_with())
            .into_stream();
        let mut result = Vec::with_capacity(limit);

        for _ in 0..limit {
            if let Some(element) = stream.next() {
                let (seq, value) = element;
                result.push(
                    self.to_sequence_description(
                        &unsafe { String::from_utf8_unchecked(seq.to_vec()) },
                        self.value_to_string
                            .get(&value)
                            .expect("value previously mapped"),
                    ),
                );
            } else {
                return result;
            }
        }

        result
    }

    pub(super) fn index_char(&mut self, value: char) -> Result<()> {
        let mut buffer = [0; 88];
        let pstr = PSTR::from_raw(buffer.as_mut_ptr());
        let mut errcode = UErrorCode::default();
        let len = unsafe { u_charName(value as i32, U_EXTENDED_CHAR_NAME, pstr, 88, &mut errcode) };

        assert!(len.is_positive());
        let name = unsafe { pstr.to_string() }.map_err(fail)?;

        assert_eq!(len as usize, name.len());
        self.char_to_name.insert(value, name.to_string());

        Ok(())
    }

    pub(super) fn filter_sequence(
        &self,
        tokens: Vec<String>,
        limit: usize,
        languages: &[SupportedLocale],
    ) -> Vec<bindings::SequenceDescription> {
        let mut result_map = HashMap::new();

        // prioritize exact character match
        for lang in languages {
            let annotations = self.annotations.get(lang).unwrap();
            annotations
                .iter()
                .filter(|pair| tokens.iter().any(|t| pair.char.contains(t)))
                .for_each(|pair| {
                    if !result_map.contains_key(&pair.char.to_string()) {
                        result_map.insert(pair.char.to_string(), pair.desc.to_string());
                    }
                });

            if result_map.len() >= limit {
                return self.process_map(&result_map);
            }
        }

        // search in descriptions
        for lang in languages {
            let annotations = self.annotations.get(lang).unwrap();
            annotations
                .iter()
                .filter(|pair| tokens.iter().all(|t| pair.desc.contains(t)))
                .for_each(|pair| {
                    if !result_map.contains_key(&pair.char.to_string()) {
                        result_map.insert(pair.char.to_string(), pair.desc.to_string());
                    }
                });

            if result_map.len() >= limit {
                return self.process_map(&result_map);
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

    fn to_sequence_description(
        &self,
        seq: &str,
        value: &MappedString,
    ) -> bindings::SequenceDescription {
        bindings::SequenceDescription {
            sequence: seq.into(),
            result: value.to_string().into(),
            description: match value {
                MappedString::Basic(c) => self
                    .char_to_name
                    .get(&c.chars().next().unwrap())
                    .unwrap()
                    .to_string()
                    .into(),
                MappedString::Extra(s) => s.to_string().into(),
            },
        }
    }
}
