use std::collections::HashMap;

use keys::keys::{Key, KeyIndex};

mod matrix;
mod vial;

use s_expression::Expr;
pub use vial::{Item as VialItem, Vial};

#[derive(Debug, Default)]
pub struct Keyboard {
    pub matrix: matrix::Matrix,
    pub vial: Vial,
    pub source: HashMap<Key, KeyIndex>,
    pub meta: String,
}

pub fn parse_keymap(lst: &Vec<Expr>) -> Result<HashMap<Key, KeyIndex>, String> {
    lst.iter()
        .enumerate()
        .try_fold(HashMap::with_capacity(lst.len()), |mut acc, (i, expr)| {
            let key: Key = expr
                .atom()?
                .parse()
                .map_err(|_| format!("Unknown key {}", expr))?;
            if acc
                .insert(key, i.try_into().map_err(|_| format!("Parse error"))?)
                .is_some()
            {
                Err(format!("Key {:?} duplicate", key))
            } else {
                Ok(acc)
            }
        })
}

pub fn parse(content: &str) -> Result<Keyboard, String> {
    let (raw, meta) = if let Some((before, after)) = content.split_once("---") {
        (after.trim(), before.trim())
    } else {
        (content, "")
    };
    let raw = format!("({})", raw);
    let value = s_expression::from_str(raw.as_str()).map_err(|_| "Parse error".to_string())?;
    let mut keyboard = Keyboard::default();
    keyboard.meta = meta.to_string();
    value.list()?.iter().try_for_each(|i| {
        let lst = i.list()?;
        let fun = lst.first().ok_or("Expected name")?.atom()?;
        match fun {
            "matrix" => {
                keyboard.matrix = matrix::parse(&lst[1..])?;
                Ok(())
            }
            "vial" => {
                keyboard.vial = vial::parse(&lst[1..])?;
                Ok(())
            }
            "source" => {
                keyboard.source = parse_keymap(&lst[1..].to_vec())?;
                Ok(())
            }
            _ => Err(format!("Unexpected {}", fun)),
        }
    })?;
    Ok(keyboard)
}
