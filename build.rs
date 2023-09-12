use std::{os::windows::process::CommandExt, process::Command};

fn main() {
    let xcompose = r#"cd 3rd-party
git clone --filter=blob:none --no-checkout https://github.com/kragen/xcompose
cd xcompose
git sparse-checkout init --cone
git checkout master
"#;
    let loc = Command::new("powershell")
        .arg("/C")
        .raw_arg(xcompose)
        .output()
        .unwrap();
    if !loc.status.success() {
        panic!("Command executed, failing error code");
    }
    println!("{}", String::from_utf8(loc.stdout).unwrap());

    let compose_pre = r#"cd 3rd-party
git clone --filter=blob:none --no-checkout https://gitlab.freedesktop.org/xorg/lib/libx11.git
cd libx11
git sparse-checkout init --cone
git checkout tags/libX11-1.8.6
git sparse-checkout set nls/en_US.UTF-8
"#;
    let loc = Command::new("powershell")
        .arg("/C")
        .raw_arg(compose_pre)
        .output()
        .unwrap();
    if !loc.status.success() {
        panic!("Command executed, failing error code");
    }
    println!("{}", String::from_utf8(loc.stdout).unwrap());

    let wincompose = r#"cd 3rd-party
git clone --filter=blob:none --no-checkout https://github.com/samhocevar/wincompose.git
cd wincompose
git sparse-checkout init --cone
git checkout tags/v0.9.11
git sparse-checkout set src/wincompose/rules
"#;
    let loc = Command::new("powershell")
        .arg("/C")
        .raw_arg(wincompose)
        .output()
        .unwrap();
    if !loc.status.success() {
        panic!("Command executed, failing error code");
    }
    println!("{}", String::from_utf8(loc.stdout).unwrap());
}
