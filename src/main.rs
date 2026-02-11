mod layout;
use std::collections::HashMap;

use keys::keys::{Key, KeyIndex};
use layout::Layout;

fn main() -> Result<(), String> {
    env_logger::init();
    let content = std::fs::read_to_string("./layout.txt").map_err(|e| e.to_string())?;
    let layout: Layout = content.parse()?;

    let keys_by_index: HashMap<KeyIndex, Key> = layout.keys.iter().map(|(k, i)| (*i, *k)).collect();
    layout.layers.values().for_each(|l| {
        println!("Layer {}", l.name);
        let mut keys: Vec<_> = l.keys.iter().collect();
        keys.sort_by_key(|(k, _)| *k);
        for (k, a) in keys {
            println!("{:?} => {:?}", keys_by_index.get(k).unwrap(), a);
        }
        println!();
    });

    Ok(())
}
