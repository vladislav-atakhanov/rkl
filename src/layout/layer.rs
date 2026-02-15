use crate::layout::action::Action;
use keys::keys::{Key, KeyIndex};
use s_expression::Expr::{self, *};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone)]
pub struct Override {
    pub key: Key,
    pub mods: Vec<Key>,
    pub action: Action,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Keymap {
    En,
    Ru,
}

impl Default for Keymap {
    fn default() -> Self {
        Keymap::En
    }
}
impl FromStr for Keymap {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "ru" => Self::Ru,
            "en" => Self::En,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct Layer {
    pub name: String,
    pub parent: String,
    pub keys: HashMap<KeyIndex, Action>,
    pub overrides: Vec<Override>,
    pub index: usize,
    pub keymap: Keymap,
}

impl Layer {
    pub fn child(&self, name: String, index: usize) -> Self {
        Self {
            name: name,
            parent: self.name.clone(),
            keys: self.keys.clone(),
            overrides: self.overrides.clone(),
            index: index,
            keymap: self.keymap.clone(),
        }
    }
    pub fn from_keyboard(source: &HashMap<Key, KeyIndex>) -> Self {
        Self {
            name: "src".to_string(),
            parent: String::new(),
            keys: source
                .iter()
                .map(|(k, v)| (*v, Action::Tap(k.clone())))
                .collect(),
            overrides: Default::default(),
            index: 0,
            keymap: Default::default(),
        }
    }
    pub fn from_def(params: &[Expr<'_>], index: usize) -> Result<Self, String> {
        let (name, parent, actions) = Self::get_name(params)?;
        Ok(Self {
            name: name.to_string(),
            parent: parent.to_string(),
            keys: actions.iter().enumerate().try_fold(
                HashMap::with_capacity(actions.len()),
                |mut acc, (i, e)| {
                    acc.insert(
                        i.try_into().map_err(|_| "Parse error".to_string())?,
                        Action::from_expr(e)?,
                    );
                    Ok::<HashMap<KeyIndex, Action>, String>(acc)
                },
            )?,
            overrides: Default::default(),
            keymap: Default::default(),
            index: index,
        })
    }

    pub fn get_dependencies(&self) -> Vec<&str> {
        let mut layers: Vec<_> = self
            .keys
            .values()
            .flat_map(|k| k.layer_while_held_names())
            .filter(|x| *x != self.name)
            .collect();
        layers.dedup();
        layers
    }

    pub fn get_name<'a>(
        params: &'a [Expr<'a>],
    ) -> Result<(&'a str, &'a str, &'a [Expr<'a>]), String> {
        let [name, params @ ..] = params else {
            return Err("Syntax error".to_string());
        };
        let (name, parent) = match name {
            Atom(x) => match *x {
                "default" => ("default", "src"),
                "src" => return Err("Cannot override src layer".to_string()),
                name => (name, "default"),
            },
            List(xs) => {
                if let [Atom(name), Atom(parent)] = xs.as_slice() {
                    (*name, *parent)
                } else {
                    return Err("Syntax error".to_string());
                }
            }
        };
        Ok((
            name,
            parent,
            match params {
                [List(x)] => x,
                _ => params,
            },
        ))
    }
    pub fn from_map(
        params: &[Expr<'_>],
        index_by_key: &HashMap<Key, KeyIndex>,
    ) -> Result<Self, String> {
        let (name, parent, params) = Self::get_name(params)?;
        Ok(Layer {
            name: name.to_string(),
            parent: parent.to_string(),
            keys: params.chunks(2).into_iter().try_fold(
                HashMap::with_capacity(params.len()),
                |mut acc, v| {
                    let [Atom(key), expr] = v else {
                        return Err("Syntax error".to_string());
                    };
                    let src: Key = key.parse().map_err(|_| format!("Unknown key {:?}", key))?;
                    let index = index_by_key
                        .get(&src)
                        .ok_or(format!("Index for {:?} not found", src))?;
                    let action = Action::from_expr(expr)?;
                    acc.insert(*index, action);
                    Ok(acc)
                },
            )?,
            overrides: Default::default(),
            keymap: Default::default(),
            index: 0,
        })
    }
}
