#![feature(path_add_extension)]

use std::{env, fs, path::PathBuf};

use libml::game::networksave::NetworkSave;

fn main() {
    let path = env::args().skip(1).collect::<Vec<_>>().join(" ");
    let network_save = NetworkSave::load(&path).expect("Couldn't load network save");
    
    let serialized_network = serde_json::to_string_pretty(&network_save.network).expect("Couldn't serialize network");
    
    let serialized_path = PathBuf::from(path)
        .with_added_extension("netdump.json");

    fs::write(serialized_path, serialized_network).expect("Couldn't save serialized network");
}
