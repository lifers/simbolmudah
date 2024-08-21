use std::{fs::File, io::Write};

use libsimbolmudah_cldr::{parse_cldr_annotations, SupportedLocale};

fn generate_bindgen() {
    println!("cargo:rerun-if-changed=src/libsimbolmudah.idl");
    let metadata_dir = format!("{}\\System32\\WinMetadata", env!("windir"));
    let mut command = std::process::Command::new("midlrt.exe");

    command.args([
        "/winrt",
        "/nomidl",
        "/h",
        "nul",
        "/metadata_dir",
        &metadata_dir,
        "/reference",
        &format!("{metadata_dir}\\Windows.Foundation.winmd"),
        "/winmd",
        "libsimbolmudah.winmd",
        "src/libsimbolmudah.idl",
    ]);

    if !command.status().unwrap().success() {
        panic!("failed to run midlrt.exe");
    }

    if let Err(error) = windows_bindgen::bindgen([
        "--in",
        "libsimbolmudah.winmd",
        &metadata_dir,
        "--out",
        "src/bindings.rs",
        "--filter",
        "LibSimbolMudah",
        "--config",
        "implement",
    ]) {
        panic!("failed to run windows_bindgen: {}", error);
    }
}

fn encode_annotations() {
    println!("cargo:rerun-if-changed=cldr-annotations-derived-full");
    for locale in [
        SupportedLocale::en,
        SupportedLocale::id,
        SupportedLocale::fr,
        SupportedLocale::jv,
    ] {
        let annotations = parse_cldr_annotations(
            locale,
            &format!("cldr-annotations-derived-full/{}/annotations.json", locale),
        );
        let mut file = File::create(format!(
            "cldr-annotations-derived-full/processed/annotations-{}.rkyv",
            locale
        ))
        .expect("file created successfully");
        let bytes = rkyv::to_bytes::<_, 1024>(&annotations).expect("bytes created successfully");
        file.write_all(&bytes).expect("bytes written successfully");
    }
}

fn main() {
    let is_debug = std::env::var("PROFILE").unwrap() == "debug";
    if is_debug {
        std::env::set_var("RUST_BACKTRACE", "full");
    }
    let headers_enabled = std::env::var("CARGO_FEATURE_HEADERS").is_ok();
    if headers_enabled {
        generate_bindgen();
    } else {
        println!("cargo:warning=The 'headers' feature is not enabled. The generated bindings will not be available.");
    }

    encode_annotations();
}
