fn main() {
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