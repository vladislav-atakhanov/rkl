use keys::Key;
use s_expression::{Expr, Expr::*};

#[rustfmt::skip]
#[derive(Debug)]
struct Item {
    key: Key,
    x: f32, y: f32, w: f32, h: f32,
    r: f32, rx: f32, ry: f32
}

#[derive(Debug, Default)]
pub struct Matrix(Vec<Item>);

fn parse_item(row: &[&str]) -> Option<Item> {
    match row.len() {
        5 => {
            let [key, x, y, w, h] = <[&str; 5]>::try_from(row).ok()?;
            Some(Item {
                key: key.parse().ok()?,
                x: x.parse().ok()?,
                y: y.parse().ok()?,
                w: w.parse().ok()?,
                h: h.parse().ok()?,
                r: 0.0,
                rx: 0.0,
                ry: 0.0,
            })
        }
        8 => {
            let [key, x, y, w, h, r, rx, ry] = <[&str; 8]>::try_from(row).ok()?;
            Some(Item {
                key: key.parse().ok()?,
                x: x.parse().ok()?,
                y: y.parse().ok()?,
                w: w.parse().ok()?,
                h: h.parse().ok()?,
                r: r.parse().ok()?,
                rx: rx.parse().ok()?,
                ry: ry.parse().ok()?,
            })
        }
        _ => None,
    }
}

pub fn parse<'a>(items: &[Expr<'a>]) -> Result<Matrix, String> {
    let mut matrix: Vec<Item> = Vec::with_capacity(items.len());
    items
        .iter()
        .try_for_each(|x| {
            if let List(row) = x {
                let row = row
                    .iter()
                    .filter_map(|el| if let Atom(a) = el { Some(*a) } else { None });
                let row: Vec<&str> = row.collect();
                if let Some(m) = parse_item(row.as_slice()) {
                    matrix.push(m);
                    Ok(())
                } else {
                    Err(format!("Cannot parse {} items", row.len()))
                }
            } else {
                Err("Expected list".to_string())
            }
        })
        .map_err(|_| "".to_string())?;
    Ok(Matrix(matrix))
}
