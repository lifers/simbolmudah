use std::{fs, io::Write};

use brotli::{
    enc::{backward_references::BrotliEncoderMode, BrotliEncoderParams},
    CompressorWriter,
};

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

fn compress_annotations() {
    println!("cargo:rerun-if-changed=../git-deps/cldr");

    let params = BrotliEncoderParams {
        mode: BrotliEncoderMode::BROTLI_MODE_TEXT,
        quality: 11,
        lgwin: 22,
        ..Default::default()
    };

    std::thread::scope(|s| {
        for locale in ["en", "fr", "id", "jv"] {
            let locale_clone = locale.to_string();
            let params_clone = params.clone();
            s.spawn(move || {
                let outpath = format!("cldr/{}-annotations.xml.br", locale_clone);
                let mut output = fs::File::create(outpath).unwrap();
                let mut compressed =
                    CompressorWriter::with_params(&mut output, 4096, &params_clone);
                let inpath = format!("../git-deps/cldr/common/annotations/{}.xml", locale_clone);
                let input = fs::read(inpath).unwrap();
                compressed.write_all(input.as_slice()).unwrap();
            });

            let locale_clone = locale.to_string();
            let params_clone = params.clone();
            s.spawn(move || {
                let outpath = format!("cldr/{}-annotationsDerived.xml.br", locale_clone);
                let mut output = fs::File::create(outpath).unwrap();
                let mut compressed =
                    CompressorWriter::with_params(&mut output, 4096, &params_clone);
                let inpath = format!(
                    "../git-deps/cldr/common/annotationsDerived/{}.xml",
                    locale_clone
                );
                let input = fs::read(inpath).unwrap();
                compressed.write_all(input.as_slice()).unwrap();
            });
        }
    });
}

fn compress_x11_files() {
    println!("cargo:rerun-if-changed=../git-deps/libX11");
    let params = BrotliEncoderParams {
        mode: BrotliEncoderMode::BROTLI_MODE_TEXT,
        quality: 11,
        lgwin: 22,
        ..Default::default()
    };

    std::thread::scope(|s| {
        let params_clone = params.clone();
        s.spawn(move || {
            let outpath = "x11-defs/Compose.pre.br";
            let mut output = fs::File::create(outpath).unwrap();
            let mut compressed =
                CompressorWriter::with_params(&mut output, 4096, &params_clone);
            let inpath = "../git-deps/libX11/nls/en_US.UTF-8/Compose.pre";
            let input = fs::read(inpath).unwrap();
            compressed.write_all(input.as_slice()).unwrap();
        });

        let params_clone = params.clone();
        s.spawn(move || {
            let outpath = "x11-defs/keysymdef.h.br";
            let mut output = fs::File::create(outpath).unwrap();
            let mut compressed =
                CompressorWriter::with_params(&mut output, 4096, &params_clone);
            let inpath = "../git-deps/xorgproto/include/X11/keysymdef.h";
            let input = fs::read(inpath).unwrap();
            compressed.write_all(input.as_slice()).unwrap();
        });
    })
}

fn main() {
    let is_debug = std::env::var("PROFILE").unwrap() == "debug";
    if is_debug {
        std::env::set_var("RUST_BACKTRACE", "full");
    }
    let headers_enabled = std::env::var("CARGO_FEATURE_BUILD_HEADERS").is_ok();
    if headers_enabled {
        generate_bindgen();
    } else {
        println!("cargo:warning=The 'headers' feature is not enabled. The generated bindings will not be available.");
    }

    let annotations_enabled = std::env::var("CARGO_FEATURE_BUILD_ANNOTATIONS").is_ok();
    if annotations_enabled {
        compress_annotations();
    } else {
        println!("cargo:warning=The 'annotations' feature is not enabled. The compressed annotations will not be available.");
    }

    let x11_defs_enabled = std::env::var("CARGO_FEATURE_BUILD_X11_DEFS").is_ok();
    if x11_defs_enabled {
        compress_x11_files();
    } else {
        println!("cargo:warning=The 'x11-defs' feature is not enabled. The compressed X11 files will not be available.");
    }
}
