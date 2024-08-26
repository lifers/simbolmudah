use std::{fmt::Display, io::Read, rc::Rc};

use quick_xml::de::from_str;
use serde::Deserialize;
use windows::{
    Foundation::Uri, Globalization::Language, Storage::StorageFile, Win32::Foundation::E_INVALIDARG,
};

use crate::utils::functions::fail;

#[derive(Deserialize, Debug)]
struct TopLevel {
    annotations: List,
}

#[derive(Deserialize, Debug)]
struct List {
    annotation: Vec<Annotation>,
}

#[derive(Deserialize, Debug)]
pub(super) struct Annotation {
    #[serde(rename = "@cp")]
    pub(super) cp: String,

    #[serde(rename = "@type")]
    pub(super) r#type: Option<String>,

    #[serde(rename = "$value")]
    pub(super) text: String,
}

#[derive(Debug)]
pub(super) struct AnnotationPair {
    pub(super) char: Rc<str>,
    pub(super) desc: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(non_camel_case_types)]
pub(super) enum SupportedLocale {
    en,
    id,
    fr,
    jv,
}

impl Display for SupportedLocale {
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

pub(super) fn load_annotation_file(application_uri: &str) -> windows_core::Result<Vec<Annotation>> {
    let path =
        StorageFile::GetFileFromApplicationUriAsync(&Uri::CreateUri(&application_uri.into())?)?
            .get()?
            .Path()?
            .to_string();

    let mut file = std::fs::File::open(path).map_err(fail)?;
    let mut input = brotli_decompressor::Decompressor::new(&mut file, 4096);
    let mut buf = String::new();
    let _num = input.read_to_string(&mut buf).map_err(fail)?;
    let object: TopLevel = from_str(&buf).map_err(fail)?;
    Ok(object.annotations.annotation)
}
