mod matrix;
mod vial;

use keys::keys::{Key, KeyIndex};
pub use matrix::{Item as MatrixItem, Matrix, parse as parse_matix};
use s_expression::Expr;
use std::collections::HashMap;
pub use vial::{Item as VialItem, Vial, parse as parse_vial};

#[derive(Debug, Default)]
pub struct Keyboard {
    pub matrix: Matrix,
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

pub fn parse(keyboard: &str) -> Result<Keyboard, String> {
    let content = match keyboard {
        "imperial44" => include_str!("keyboards/imperial44.txt"),
        _ => return Err(format!("Keyboard {:?} not found", keyboard)),
    };

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
            "defmatrix" => {
                keyboard.matrix = parse_matix(&lst[1..])?;
                Ok(())
            }
            "defvial" => {
                keyboard.vial = parse_vial(&lst[1..])?;
                Ok(())
            }
            "defsrc" => {
                keyboard.source = parse_keymap(&lst[1..].to_vec())?;
                Ok(())
            }
            _ => Err(format!("Unexpected {}", fun)),
        }
    })?;
    Ok(keyboard)
}
