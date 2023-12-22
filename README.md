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
  - [x] Parse keysym definitions from keysymdef.h
  - [x] Parse basic compose sequences from en_US.UTF-8/Compose.pre
  - [x] Connect the two and generate mappings from keysym sequence to the resulting Unicode
  - [x] Rework the compose search engine to use the mappings
- [ ] Unicode explicit code input mode
- [ ] Popup window for hints
  - [ ] Display a small popup near the cursor for mode indicator (similar to Windows' autocomplete)
  - [ ] Unicode name search mode basic frontend
- [ ] Unicode name search mode
  - [ ] Use fuzzy text search engine
  - [ ] Implement recommendations
  - [ ] Implement Emoji variants
- [ ] Tray menu
  - [ ] Add exit button
  - [ ] Add hover info
- [ ] Settings UI
- [ ] Custom compose rules
  - [ ] Custom compose key
- [ ] Internationalisation
  - [ ] Make it work on non-US keyboards (VERY HARD)
  - [ ] Add translations for UI
