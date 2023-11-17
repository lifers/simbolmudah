use regex::Regex;
use std::{cell::RefCell, collections::HashMap};

thread_local! {
    static GENERAL_KEYSYM: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new())
}

fn get_general_keysym(filename: &str) {
    let re = Regex::new(
        r"^\#define XK_([a-zA-Z_0-9]+)\s+0x([0-9a-f]+)\s*\/\* U\+([0-9A-F]{4,6}) (.*) \*\/\s*$",
    )
    .unwrap();

    let hay = "#define XK_KP_9                          0xffb9  /*<U+0039 DIGIT NINE>*/";
    let Some(name) = re.captures(hay) else { return };
    assert_eq!(
        "#define XK_KP_9                          0xffb9  /*<U+0039 DIGIT NINE>*/",
        &name[0]
    );
    assert_eq!("KP_9", &name[1]);
    assert_eq!("ffb9", &name[2]);
    assert_eq!("009", &name[3]);
}

#[cfg(test)]
mod tests {
    use super::get_general_keysym;

    #[test]
    fn valid_regex() {
        get_general_keysym("filename");
    }
}
