use keys::Key;
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
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub fn parse<'a>(items: &[Expr<'a>]) -> Result<Vial, String> {
    let mut vial = Vial(HashMap::new());
    items.iter().try_for_each(|x| {
        let row = match x {
            List(r) => r.iter().filter_map(|e| match e {
                Atom(s) => Some(*s),
                _ => None,
            }),
            _ => return Err("Expected list".to_string()),
        };
        let row: Vec<&str> = row.collect();
        let first = row.first().ok_or("Key not found".to_string())?;
        let key: Key = first
            .parse()
            .map_err(|_| format!("Unknown key {}", first))?;
        let item = if row.len() == 3 {
            Some(Item(
                row[1]
                    .parse()
                    .map_err(|_| format!("Unknown value {}", row[1]))?,
                row[2]
                    .parse()
                    .map_err(|_| format!("Unknown value {}", row[2]))?,
                Type::Key,
            ))
        } else if row.len() == 4 {
            Some(Item(
                row[1]
                    .parse()
                    .map_err(|_| format!("Unknown value {}", row[1]))?,
                row[2]
                    .parse()
                    .map_err(|_| format!("Unknown value {}", row[2]))?,
                match row[3] {
                    "e" => Type::Encoder,
                    _ => Type::Key,
                },
            ))
        } else {
            None
        };
        match vial.add(key, item.ok_or(format!("Unexpected {:?}", row))?) {
            None => {}
            _ => return Err(format!("Key {:?} already in map", key)),
        }
        Ok(())
    })?;
    if items.len() == vial.len() {
        Ok(vial)
    } else {
        Err("Some keys duplicates".to_string())
    }
}
