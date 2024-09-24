# simbolmudah
Makes it easier for Windows users to input symbols, accented characters, and emojis.
This is inspired by the Compose Key mechanism from Freedesktop's X project.

## Roadmap
- [x] Basic mechanism for composing key
  - [x] Translate keyboard input into internal representation
  - [x] Translate internal representation into output
  - [x] Store and release intermediate keys during composing operation
  - [x] Example compose sequence for basic usage
- [x] Compose mode
  - [x] Get compose sequence from Freedesktop's X project
  - [x] Parse keysym definitions from keysymdef.txt
  - [x] Parse basic compose sequences from en_US.UTF-8/Compose.pre
  - [x] Connect the two and generate mappings from keysym sequence to the resulting Unicode
  - [x] Rework the compose search engine to use the mappings
- [x] Unicode explicit code input mode
- [x] Popup window for hints
  - [x] Display a small popup near the cursor for mode indicator (similar to Windows' autocomplete)
  - [x] Unicode name search mode basic frontend
- [x] Unicode name search mode
  - [ ] Use fuzzy text search engine
  - [x] Implement recommendations
  - [ ] Implement specific UI for Emoji variants
- [x] Tray menu
  - [x] Add exit button
  - [x] Add hover info
- [x] Settings UI
- [x] In-app user guide
- [ ] Custom compose rules
  - [ ] Custom compose key
- [ ] Internationalisation
  - [ ] Make it work on non-US keyboards (VERY HARD)
  - [ ] Add translations for UI
