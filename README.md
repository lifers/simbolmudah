# simbolmudah
An experimental [compose key](https://en.wikipedia.org/wiki/Compose_key) implementation on Windows.
Type mathematical symbols, emojis, and Unicode symbols in a few keystrokes.

![hero2](https://github.com/user-attachments/assets/c83dc0d0-647f-4b02-82bc-098524ace9cc)

## Features
- Compose key that runs in background
- Unicode codepoint input mode
- Search symbol by its name

![hero1](https://github.com/user-attachments/assets/1a416700-dce3-47a7-9272-80936d8e3d6e)

## Support
This software is tested on Windows 11. There is no guarantee that this will work on Windows 10 or lower.
So far the UI is English-only with future plans of translation. For symbol names, we support English, French, Indonesian, and Javanese.

## Build
In order to build this program, you need a working installation of:
- Visual Studio 2022 (>=17.11.4, community or more)
- Rust toolchain (>=1.81.0)

Follow these steps:
1. Open _x64 Native Tools Command Prompt for VS 2022_ and go to `simbolmudah/` subdirectory
2. Build the Rust library and wait for it to finish.
```{powershell}
cargo build --release --features=build-headers,build-annotations,build-x11-defs
```
3. Open `simbolmudah.sln` in Visual Studio 2022. Set the target to `Release` and `x64`, then choose `Build` ⇒ `Build Solution`.
   
