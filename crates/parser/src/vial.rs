use keys::keys::Key;
use s_expression::{Expr, Expr::*};
use std::collections::HashMap;

#[derive(Debug)]
enum Type {
    Key,
    Encoder,
}

#[derive(Debug)]
struct Item(u8, u8, Type);

#[derive(Debug, Default)]
pub struct Vial(HashMap<Key, Item>);

impl Vial {
    fn add(&mut self, key: Key, item: Item) -> Option<Item> {
        self.0.insert(key, item)
    }
}

pub fn parse<'a>(items: &[Expr<'a>]) -> Result<Vial, String> {
    let mut vial = Vial(HashMap::new());
    items.iter().try_for_each(|x| {
        let row = x.list()?.iter().filter_map(|e| match e {
            Atom(s) => Some(*s),
            _ => None,
        });
        let row: Vec<&str> = row.collect();
        let first = row.first().ok_or("Key not found".to_string())?;
        let key: Key = first
            .parse()
            .map_err(|_| format!("Unknown key {}", first))?;
        let item = match row.as_slice() {
            [_, a, b] | [_, a, b, _] => {
                let a = a.parse().map_err(|_| format!("Unknown value {}", a))?;
                let b = b.parse().map_err(|_| format!("Unknown value {}", b))?;
                let t = match row.get(3) {
                    Some(&"e") => Type::Encoder,
                    _ => Type::Key,
                };
                Some(Item(a, b, t))
            }
            _ => None,
        };

        match vial.add(key, item.ok_or(format!("Unexpected {:?}", row))?) {
            None => {}
            _ => return Err(format!("Key {:?} already in map", key)),
        }
        Ok(())
    })?;
    Ok(vial)
}
