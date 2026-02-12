use keys::keys::{Key, KeyIndex};
use parser::Keyboard;
use s_expression::Expr::*;
use std::{collections::HashMap, str::FromStr};

mod action;
mod layer;
mod preprocess;
mod template;
mod unwrap;
pub use action::Action;
pub use layer::Layer;
pub use layer::Override;
use preprocess::preprocess;

#[derive(Debug, Default)]
pub struct Layout {
    pub layers: HashMap<String, Layer>,
    pub keyboard: Keyboard,
}
impl Layout {
    fn new() -> Self {
        Self::default()
    }
    fn prepare_layers(&mut self, aliases: &HashMap<String, Action>) -> Result<(), String> {
        let layer_names: Vec<String> = self.layers.keys().cloned().collect();

        for name in &layer_names {
            let layer = self.layers.get_mut(name).unwrap();
            for action in layer.keys.values_mut() {
                *action = action.resolve_aliases(aliases)?;
            }
        }

        for name in &layer_names {
            let layer = self.layers.get(name).unwrap();
            let parent_name = layer.parent.clone();
            let updates: Vec<_> = layer
                .keys
                .iter()
                .filter_map(|(&key, action)| {
                    if !matches!(action, Action::Transparent) {
                        return None;
                    }

                    let mut current = parent_name.as_str();

                    loop {
                        let parent = self.layers.get(current)?;
                        match parent.keys.get(&key) {
                            Some(a) if !matches!(a, Action::Transparent) => {
                                break Some((key, a.clone()));
                            }
                            _ => current = &parent.parent,
                        }
                    }
                })
                .collect();

            if let Some(layer) = self.layers.get_mut(name) {
                for (key, action) in updates {
                    layer.keys.insert(key, action);
                }
            }
        }
        self.layers.remove("src");
        Ok(())
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
            Ok(parent.child(name))
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
                        layout.keyboard = parser::parse(content.as_str())?;
                        let src = Layer::from_keyboard(&layout.keyboard.source);
                        layout.layers.insert(src.name.to_string(), src);
                    }
                    "deflayer" => {
                        let layer = Layer::from_def(params)?;
                        let keys = &layout.keyboard.source;
                        if layer.keys.len() != keys.len() {
                            return Err(format!(
                                "Syntax error: expected {}, found {} ({})",
                                keys.len(),
                                layer.keys.len(),
                                name
                            ));
                        }
                        layout.layers.insert(layer.name.to_string(), layer);
                    }
                    "deflayermap" => {
                        let layer = Layer::from_map(params, &layout.keyboard.source)?;
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
                                        return Err(format!("Syntax error: {:?}", x));
                                    };
                                    let action = Action::from_expr(expr)?;
                                    Ok((name.to_string(), action))
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                    }
                    "defoverride" => {
                        let (name, parent, params) = Layer::get_name(params)?;
                        let mut layer = layout.layer_from(parent.to_string(), name.to_string())?;
                        layer.overrides = params
                            .chunks(2)
                            .map(|x| {
                                let [Atom(src), expr] = x else {
                                    println!("{:?}", x);
                                    return Err(format!("Syntax error: {:?}", x));
                                };
                                let Action::Multi(src) = Action::from_expr(&Atom(*src))? else {
                                    return Err(format!("Expected hotkey, found {:?}", src));
                                };
                                let src: Vec<_> = src
                                    .into_iter()
                                    .map(|action| match action {
                                        Action::Tap(key) => Ok(key),
                                        x => Err(format!("Expected tap, found {:?}", x)),
                                    })
                                    .collect::<Result<_, _>>()?;

                                let [mods @ .., key] = src.as_slice() else {
                                    return Err(format!("Expected hotkey, found {:?}", src));
                                };

                                check_all_with(mods, |k| k.is_modifier())
                                    .map_err(|k| format!("Key {:?} is not modifier", k))?;

                                Ok(Override {
                                    key: layout
                                        .keyboard
                                        .source
                                        .get(key)
                                        .ok_or(format!("Key {:?} is not found", key))?
                                        .clone(),
                                    action: Action::from_expr(expr)?,
                                    mods: mods.to_vec(),
                                })
                            })
                            .collect::<Result<_, _>>()?;
                        layout.layers.insert(layer.name.to_string(), layer);
                    }

                    _ => return Err(format!("Unexpected {}", name)),
                }
                Ok(())
            })?;
        layout.prepare_layers(&aliases)?;
        Ok(layout)
    }
}

fn check_all_with<T, F>(src: &[T], predicate: F) -> Result<(), &T>
where
    F: Fn(&T) -> bool,
{
    src.iter()
        .find(|item| !predicate(item))
        .map_or(Ok(()), |bad| Err(bad))
}
