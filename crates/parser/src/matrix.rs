use keys::keys::Key;
use s_expression::Expr;

#[allow(unused)]
#[rustfmt::skip]
#[derive(Debug)]
pub struct Item {
    key: Key,
    x: f32, y: f32, w: f32, h: f32,
    r: f32, rx: f32, ry: f32
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Matrix(Vec<Item>);

fn parse_item(row: &[&str]) -> Result<Item, String> {
    match row.len() {
        5 => {
            let [key, x, y, w, h] =
                <[&str; 5]>::try_from(row).map_err(|_| "Unpack error".to_string())?;
            Ok(Item {
                key: key.parse().map_err(|_| format!("Unknown key {}", key))?,
                x: x.parse().map_err(|_| "parse float".to_string())?,
                y: y.parse().map_err(|_| "parse float".to_string())?,
                w: w.parse().map_err(|_| "parse float".to_string())?,
                h: h.parse().map_err(|_| "parse float".to_string())?,
                r: 0.0,
                rx: 0.0,
                ry: 0.0,
            })
        }
        8 => {
            let [key, x, y, w, h, r, rx, ry] =
                <[&str; 8]>::try_from(row).map_err(|_| "Unpack error".to_string())?;
            Ok(Item {
                key: key.parse().map_err(|_| format!("Unknown key {}", key))?,
                x: x.parse().map_err(|_| "parse float".to_string())?,
                y: y.parse().map_err(|_| "parse float".to_string())?,
                w: w.parse().map_err(|_| "parse float".to_string())?,
                h: h.parse().map_err(|_| "parse float".to_string())?,
                r: r.parse().map_err(|_| "parse float".to_string())?,
                rx: rx.parse().map_err(|_| "parse float".to_string())?,
                ry: ry.parse().map_err(|_| "parse float".to_string())?,
            })
        }
        _ => Err(format!(
            "Cannot parse {:?} items (length {})",
            row,
            row.len()
        )),
    }
}

pub fn parse<'a>(items: &[Expr<'a>]) -> Result<Matrix, String> {
    let mut matrix: Vec<Item> = Vec::with_capacity(items.len());
    items.iter().try_for_each(|x| {
        let row = x.list()?.iter().filter_map(|el| el.atom().ok());
        let row: Vec<&str> = row.collect();
        matrix.push(parse_item(row.as_slice())?);
        Ok::<(), String>(())
    })?;
    Ok(Matrix(matrix))
}
