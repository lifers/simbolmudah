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
        "/reference",
        &format!("{metadata_dir}\\Windows.Graphics.winmd"),
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
}