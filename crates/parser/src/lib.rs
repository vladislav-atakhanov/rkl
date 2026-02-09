mod matrix;
mod vial;
use s_expression::Expr::*;

#[derive(Debug, Default)]
pub struct Keyboard<'a> {
    pub matrix: matrix::Matrix,
    pub vial: vial::Vial,
    pub meta: &'a str,
}

pub fn parse<'a>(content: &'a str) -> Result<Keyboard<'a>, String> {
    let (raw, meta) = if let Some((before, after)) = content.split_once("---") {
        (after.trim(), before.trim())
    } else {
        (content, "")
    };
    let raw = format!("({})", raw);
    let value = s_expression::from_str(raw.as_str()).map_err(|_| "Parse error".to_string())?;
    let mut keyboard = Keyboard::default();
    keyboard.meta = meta;
    let root = match value {
        List(r) => r,
        _ => return Err("Expected list".to_string()),
    };
    root.iter().try_for_each(|i| {
        let lst = match i {
            List(lst) => lst,
            _ => return Err("Expected list".to_string()),
        };
        let fun = lst
            .first()
            .map_or(None, |f| match f {
                Atom(f) => Some(*f),
                _ => None,
            })
            .ok_or("Expected atom".to_string())?;
        match fun {
            "matrix" => {
                keyboard.matrix = matrix::parse(&lst[1..])?;
                Ok(())
            }
            "vial" => {
                keyboard.vial = vial::parse(&lst[1..])?;
                Ok(())
            }
            _ => Err(format!("Unexpected {}", fun)),
        }
    })?;
    Ok(keyboard)
}
