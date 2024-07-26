use crate::{fail, fail_message};
use regex::Regex;
use std::{
    cell::{LazyCell, RefCell},
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};
use windows::{
    core::{Error, Result},
    Win32::Foundation::E_FAIL,
};

use super::internal::SequenceDefinitionInternal;

// const GENERAL_REGEX_STR: &str = r"^#define XK_([a-zA-Z_0-9]+)\s+0x([0-9a-f]+)\s*(/\*.*\*/)?\s*$";
const UNICODE_REGEX_STR: &str =
    r"^#define XK_([a-zA-Z_0-9]+)\s+0x([0-9a-f]+)\s*/\*[ <(]U\+([0-9A-F]{4,6}) (.*)[ >)]\*/\s*$";

thread_local! {
    static GENERAL_KEYSYM: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
    static KEYPAD_REGEX: LazyCell<Regex> = LazyCell::new(|| Regex::new(
        r"^#define XK_(KP_[a-zA-Z_0-9]+)\s+0x([0-9a-f]+)\s*(/\*[ |<].*[ |>]\*/)?\s*$"
    ).unwrap());
    // static UNICODE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(UNICODE_REGEX_STR).unwrap());
    static DEPRECATED_REGEX: LazyCell<Regex> = LazyCell::new(|| Regex::new(
        r"^#define XK_([a-zA-Z_0-9]+)\s+0x([0-9a-f]+)\s*/\* (deprecated.*) \*/\s*$"
    ).unwrap());
    static GENERAL_REGEX: LazyCell<Regex> = LazyCell::new(|| Regex::new(
        r"^#define XK_([a-zA-Z_0-9]+)\s+0x([0-9a-f]+)\s*(/\*.*\*/)?\s*$"
    ).unwrap());
}

pub(super) struct KeySymDef {
    content: HashMap<String, char>,
}

impl KeySymDef {
    pub(super) fn new(path: &str, definition: &mut SequenceDefinitionInternal) -> Result<Self> {
        let unicode_regex =
            Regex::new(UNICODE_REGEX_STR).map_err(|e| Error::new(E_FAIL, format!("{:?}", e)))?;
        let content = get_general_keysym(&unicode_regex, path, definition)?;
        Ok(Self { content })
    }

    pub(super) fn get_key(&self, name: &str) -> Result<&char> {
        self.content
            .get(name)
            .ok_or_else(|| fail_message("Invalid keyname"))
    }
}

fn get_general_keysym(
    unicode_regex: &Regex,
    path: &str,
    definition: &mut SequenceDefinitionInternal,
) -> Result<HashMap<String, char>> {
    let file = File::open(path).map_err(fail)?;
    let reader = BufReader::new(file);

    let mut result = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(fail)?;
        if let Some(caps) = unicode_regex.captures(&line) {
            let name = caps
                .get(1)
                .ok_or_else(|| fail_message("Regex parse"))?
                .as_str();
            let value = char::from_u32(
                u32::from_str_radix(
                    caps.get(3)
                        .ok_or_else(|| fail_message("Regex parse"))?
                        .as_str(),
                    16,
                )
                .map_err(fail)?,
            )
            .ok_or_else(|| fail_message("Invalid char"))?;

            definition.index_char(value)?;
            result.insert(name.to_string(), value);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{DEPRECATED_REGEX, GENERAL_REGEX, KEYPAD_REGEX, UNICODE_REGEX_STR};
    use regex::Regex;

    #[test]
    fn general_key_regex_good() {
        let hay1 = "#define XK_VoidSymbol                  0xffffff  /* Void symbol */";
        let hay2 = "#define XK_KP_F1                         0xff91  /* PF1, KP_A, ... */";
        let hay3 = "#define XK_F3                            0xffc0";

        let Some(name1) = GENERAL_REGEX.with(|re| re.captures(hay1)) else {
            panic!()
        };
        let Some(name2) = GENERAL_REGEX.with(|re| re.captures(hay2)) else {
            panic!()
        };
        let Some(name3) = GENERAL_REGEX.with(|re| re.captures(hay3)) else {
            panic!()
        };

        assert_eq!(
            "#define XK_VoidSymbol                  0xffffff  /* Void symbol */",
            &name1[0]
        );
        assert_eq!("VoidSymbol", &name1[1]);
        assert_eq!("ffffff", &name1[2]);
        assert_eq!("/* Void symbol */", &name1[3]);

        assert_eq!(
            "#define XK_KP_F1                         0xff91  /* PF1, KP_A, ... */",
            &name2[0]
        );
        assert_eq!("KP_F1", &name2[1]);
        assert_eq!("ff91", &name2[2]);
        assert_eq!("/* PF1, KP_A, ... */", &name2[3]);

        assert_eq!("#define XK_F3                            0xffc0", &name3[0]);
        assert_eq!("F3", &name3[1]);
        assert_eq!("ffc0", &name3[2]);
        assert_eq!(None, name3.get(3));
    }

    #[test]
    fn keypad_key_regex_good() {
        let hay1 = "#define XK_KP_9                          0xffb9  /*<U+0039 DIGIT NINE>*/";
        let hay2 = "#define XK_KP_F1                         0xff91  /* PF1, KP_A, ... */";
        let hay3 = "#define XK_KP_F3                         0xff93";

        let Some(name1) = KEYPAD_REGEX.with(|re| re.captures(hay1)) else {
            panic!()
        };
        let Some(name2) = KEYPAD_REGEX.with(|re| re.captures(hay2)) else {
            panic!()
        };
        let Some(name3) = KEYPAD_REGEX.with(|re| re.captures(hay3)) else {
            panic!()
        };

        assert_eq!(
            "#define XK_KP_9                          0xffb9  /*<U+0039 DIGIT NINE>*/",
            &name1[0]
        );
        assert_eq!("KP_9", &name1[1]);
        assert_eq!("ffb9", &name1[2]);
        assert_eq!("/*<U+0039 DIGIT NINE>*/", &name1[3]);

        assert_eq!(
            "#define XK_KP_F1                         0xff91  /* PF1, KP_A, ... */",
            &name2[0]
        );
        assert_eq!("KP_F1", &name2[1]);
        assert_eq!("ff91", &name2[2]);
        assert_eq!("/* PF1, KP_A, ... */", &name2[3]);

        assert_eq!("#define XK_KP_F3                         0xff93", &name3[0]);
        assert_eq!("KP_F3", &name3[1]);
        assert_eq!("ff93", &name3[2]);
        assert_eq!(None, name3.get(3));
    }

    #[test]
    fn keypad_key_regex_bad() {
        let hay = "#define XK_F3                            0xffc0";
        KEYPAD_REGEX.with(|re| assert!(!re.is_match(hay)));
    }

    #[test]
    fn unicode_key_regex_good() {
        let unicode_regex = Regex::new(UNICODE_REGEX_STR).unwrap();
        let hay = "#define XK_Return                        0xff0d  /* U+000D CARRIAGE RETURN */";
        let Some(name) = unicode_regex.captures(hay) else {
            panic!()
        };
        assert_eq!(
            "#define XK_Return                        0xff0d  /* U+000D CARRIAGE RETURN */",
            &name[0]
        );
        assert_eq!("Return", &name[1]);
        assert_eq!("ff0d", &name[2]);
        assert_eq!("000D", &name[3]);
        assert_eq!("CARRIAGE RETURN", &name[4]);
    }

    #[test]
    fn deprecated_key_regex_good() {
        let hay1 =
            "#define XK_KP_Page_Up                    0xff9a  /* deprecated alias for KP_Prior */";
        let hay2 =
            "#define XK_dead_small_schwa              0xfe8a  /* deprecated, remove in 2025 */";
        let hay3 = "#define XK_quoteright                    0x0027  /* deprecated */";

        let Some(name1) = DEPRECATED_REGEX.with(|re| re.captures(hay1)) else {
            panic!()
        };
        let Some(name2) = DEPRECATED_REGEX.with(|re| re.captures(hay2)) else {
            panic!()
        };
        let Some(name3) = DEPRECATED_REGEX.with(|re| re.captures(hay3)) else {
            panic!()
        };

        assert_eq!(
            "#define XK_KP_Page_Up                    0xff9a  /* deprecated alias for KP_Prior */",
            &name1[0]
        );
        assert_eq!("KP_Page_Up", &name1[1]);
        assert_eq!("ff9a", &name1[2]);
        assert_eq!("deprecated alias for KP_Prior", &name1[3]);

        assert_eq!(
            "#define XK_dead_small_schwa              0xfe8a  /* deprecated, remove in 2025 */",
            &name2[0]
        );
        assert_eq!("dead_small_schwa", &name2[1]);
        assert_eq!("fe8a", &name2[2]);
        assert_eq!("deprecated, remove in 2025", &name2[3]);

        assert_eq!(
            "#define XK_quoteright                    0x0027  /* deprecated */",
            &name3[0]
        );
        assert_eq!("quoteright", &name3[1]);
        assert_eq!("0027", &name3[2]);
        assert_eq!("deprecated", &name3[3]);
    }
}
