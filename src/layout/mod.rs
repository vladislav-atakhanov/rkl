use keys::keys::{Key, KeyIndex};
use s_expression::Expr::*;
use std::{collections::HashMap, str::FromStr};

mod action;
mod layer;
mod preprocess;
mod template;
mod unwrap;
pub use layer::Layer;

use crate::layout::action::Action;
use preprocess::preprocess;

#[derive(Debug, Default)]
pub struct Layout {
    pub keys: HashMap<Key, KeyIndex>,
    pub layers: HashMap<String, Layer>,
}
impl Layout {
    fn new() -> Self {
        Self::default()
    }
    fn prepare_layers(&mut self, aliases: &HashMap<String, Action>) {
        let layer_names: Vec<String> = self.layers.keys().cloned().collect();
        for name in &layer_names {
            let layer = self.layers.get(name).unwrap();
            let parent_name = layer.parent.clone();

            let updates: Vec<(KeyIndex, Action)> = layer
                .keys
                .iter()
                .filter_map(|(key, action)| match action {
                    Action::Transparent => {
                        let mut current_parent = parent_name.clone();
                        loop {
                            let parent_layer = self.layers.get(&current_parent)?;
                            match parent_layer.keys.get(key) {
                                Some(a) if !matches!(a, Action::Transparent) => {
                                    break Some((*key, a.clone()));
                                }
                                _ => {
                                    current_parent = parent_layer.parent.clone();
                                }
                            }
                        }
                    }
                    Action::Alias(alias) => {
                        if let Some(a) = aliases.get(alias) {
                            Some((*key, a.clone()))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect();

            let layer = self.layers.get_mut(name).unwrap();
            for (key, action) in updates {
                layer.keys.insert(key, action);
            }
        }
    }
    fn layer_from(&self, parent: String, name: String) -> Result<Layer, String> {
        let Some(parent) = self
            .layers
            .get(&name)
            .or_else(|| self.layers.get(&parent))
            .or_else(|| self.layers.get(&"src".to_string()))
        else {
            return Err(format!("Layer {:?} not defined", parent));
        };
        if parent.name == name {
            Ok(parent.clone())
        } else {
            Ok(Layer {
                name: name,
                parent: parent.name.clone(),
                keys: parent.keys.clone(),
            })
        }
    }
}

impl FromStr for Layout {
    type Err = String;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = format!("({})", content);
        let expr = s_expression::from_str(content.as_str()).map_err(|_| "Parse error")?;
        let mut layout = Self::new();

        let root = preprocess(&expr)?;
        let mut aliases: HashMap<String, Action> = HashMap::new();
        root.list()?
            .iter()
            .try_for_each(|r| -> Result<(), String> {
                let [name, params @ ..] = r.list()?.as_slice() else {
                    return Err("Exprected name".to_string());
                };
                match name.atom()? {
                    "keyboard" => {
                        let [Atom(id)] = params else {
                            return Err("Syntax error".to_string());
                        };
                        let content = std::fs::read_to_string(format!("./keyboards/{}.txt", id))
                            .map_err(|e| e.to_string())?;
                        let keyboard = parser::parse(content.as_str())?;
                        layout.keys = keyboard.source;
                        let src = Layer::from_keyboard(&layout.keys);
                        layout.layers.insert(src.name.to_string(), src);
                    }
                    "deflayer" => {
                        let layer = Layer::from_def(params)?;
                        if layer.keys.len() != layout.keys.len() {
                            return Err(format!(
                                "Syntax error: expected {}, found {} ({})",
                                layout.keys.len(),
                                layer.keys.len(),
                                name
                            ));
                        }
                        layout.layers.insert(layer.name.to_string(), layer);
                    }
                    "deflayermap" => {
                        let layer = Layer::from_map(params, &layout.keys)?;
                        let mut l = layout.layer_from(layer.parent, layer.name)?;
                        l.keys.extend(layer.keys);
                        layout.layers.insert(l.name.to_string(), l);
                    }
                    "defalias" => {
                        aliases.extend(
                            params
                                .chunks(2)
                                .map(|x| {
                                    let [Atom(name), expr] = x else {
                                        println!("{:?}", x);
                                        return Err(format!("Syntax error: {:?}", x));
                                    };
                                    let action = Action::from_expr(expr)?;
                                    Ok((name.to_string(), action))
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                    }
                    "defoverride" => {}
                    _ => return Err(format!("Unexpected {}", name)),
                }
                Ok(())
            })?;
        layout.prepare_layers(&aliases);
        Ok(layout)
    }
}
