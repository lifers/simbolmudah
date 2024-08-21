use std::{fs::File, io::Read};

use windows::{Globalization::Language, Win32::Foundation::E_INVALIDARG};

#[derive(serde::Serialize, serde::Deserialize)]
#[allow(non_snake_case)]
struct Annotations {
    annotationsDerived: AnnotationsDerived,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AnnotationsDerived {
    identity: Identity,
    annotations: std::collections::HashMap<String, Annotation>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Identity {
    version: Version,
    language: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[allow(non_snake_case)]
struct Version {
    _cldrVersion: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Annotation {
    default: Option<Vec<String>>,
    tts: Option<Vec<String>>,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct AnnotationPair {
    pub char: Box<str>,
    pub desc: Box<str>,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[allow(non_camel_case_types)]
pub enum SupportedLocale {
    en,
    id,
    fr,
    jv,
}

impl std::fmt::Display for SupportedLocale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let locale_str = match self {
            SupportedLocale::en => "en",
            SupportedLocale::id => "id",
            SupportedLocale::fr => "fr",
            SupportedLocale::jv => "jv",
        };
        write!(f, "{}", locale_str)
    }
}

impl TryFrom<Language> for SupportedLocale {
    type Error = windows::core::Error;

    fn try_from(value: Language) -> Result<Self, Self::Error> {
        match value.LanguageTag()?.to_string().as_str() {
            "en" => Ok(SupportedLocale::en),
            "en-US" => Ok(SupportedLocale::en),
            "en-GB" => Ok(SupportedLocale::en),
            "en-CA" => Ok(SupportedLocale::en),
            "id" => Ok(SupportedLocale::id),
            "id-ID" => Ok(SupportedLocale::id),
            "fr" => Ok(SupportedLocale::fr),
            "fr-FR" => Ok(SupportedLocale::fr),
            "fr-CA" => Ok(SupportedLocale::fr),
            "jv" => Ok(SupportedLocale::jv),
            "jv-Java" => Ok(SupportedLocale::jv),
            _ => Err(windows::core::Error::new(
                E_INVALIDARG,
                "Unsupported locale",
            )),
        }
    }
}

impl From<&str> for SupportedLocale {
    fn from(s: &str) -> Self {
        match s {
            "en" => SupportedLocale::en,
            "id" => SupportedLocale::id,
            "fr" => SupportedLocale::fr,
            "jv" => SupportedLocale::jv,
            _ => panic!("Unsupported locale"),
        }
    }
}

pub fn parse_cldr_annotations(locale: SupportedLocale, path: &str) -> Box<[AnnotationPair]> {
    // read the JSON data into the Rust data structure
    let mut file = File::open(path).expect("file opened successfully");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("file read successfully");

    let annotations: Annotations =
        serde_json::from_str(&contents).expect("json parsed successfully");
    assert_eq!(
        annotations.annotationsDerived.identity.language,
        locale.to_string()
    );

    let mut pairs = Vec::new();
    for (emoji, annotation) in annotations.annotationsDerived.annotations {
        // generate a pair for each description in default
        if let Some(descriptions) = annotation.default {
            for desc in descriptions {
                pairs.push(AnnotationPair {
                    char: emoji.clone().into(),
                    desc: desc.into(),
                });
            }
        }
    }

    pairs.into_boxed_slice()
}
