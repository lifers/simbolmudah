// use std::fs::File;

// use fst::{automaton::Str, Automaton, IntoStreamer, Map, MapBuilder};
// use once_cell::sync::Lazy;
// use trie_rs::{Trie, TrieBuilder};

// pub static SEQUENCE_TREE: Lazy<Map<Vec<u8>>> = Lazy::new(|| {
//     let mut builder = MapBuilder::memory();
//     builder.insert("bruce", 1).unwrap();
//     builder.insert("brutus", 5).unwrap();
//     builder.insert("clarence", 2).unwrap();
//     builder.insert("stevie", 3).unwrap();

//     builder.into_map()
// });

// fn load_sequence_file(filename: &str) -> Map<Vec<u8>> {
//     let file_buffer = include_bytes!(".\\map.fst");
//     Map::new(file_buffer).unwrap()
// }
