use parser::{Keyboard, parse_vial};
use s_expression::Expr::*;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

mod action;
mod layer;
mod preprocess;
mod template;
mod unicode;
mod unwrap;
pub use action::Action;
pub use layer::{Keymap, Layer, Override};
use preprocess::preprocess;
use unicode::unicode;

#[derive(Debug, Default)]
pub struct Layout {
    pub layers: HashMap<String, Layer>,
    pub keyboard: Keyboard,
    pub keymaps: HashMap<Keymap, Action>,
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

        let new_layers: Vec<Layer> = self
            .layers
            .iter()
            .flat_map(|(_, l)| {
                let deps = l.get_dependencies();
                let mut copies = HashSet::<String>::new();
                let mut new = deps
                    .iter()
                    .filter_map(|name| {
                        let dep = self.layers.get(*name)?;
                        if dep.keymap == l.keymap {
                            return None;
                        }
                        if dep.keys.iter().all(|(_, v)| match l.keymap {
                            Keymap::En => false,
                            Keymap::Ru => !v.contains_unicode(),
                        }) {
                            return None;
                        }
                        let mut new = dep.clone();
                        let new_name = format!("{}-{:?}", dep.name, l.keymap).to_lowercase();
                        new.name = new_name.clone();

                        new.keys.values_mut().for_each(|v| {
                            v.map_layer_while_held(&|x| {
                                (x == *name).then(|| new_name.clone())
                            });
                        });

                        new.keymap = l.keymap.clone();
                        new.index += 1;
                        copies.insert(dep.name.clone());
                        Some(new)
                    })
                    .collect::<Vec<_>>();
                if new.len() > 0 {
                    let mut s = l.clone();
                    s.keys.values_mut().for_each(|v| {
                        v.map_layer_while_held(&|x| {
                            copies
                                .contains(x)
                                .then(|| format!("{}-{:?}", x, l.keymap).to_lowercase())
                        });
                    });
                    new.push(s);
                }
                new
            })
            .collect();

        new_layers.into_iter().for_each(|l| {
            self.layers.insert(l.name.to_string(), l);
        });

        self.layers.values_mut().try_for_each(|layer| {
            layer.keys.values_mut().try_for_each(|action| {
                resolve_unicode(action, &layer.keymap, &self.keymaps).map(|a| {
                    *action = a;
                })
            })?;
            layer.overrides.iter_mut().try_for_each(|o| {
                resolve_unicode(&o.action, &layer.keymap, &self.keymaps).map(|a| {
                    o.action = a.clone();
                })
            })?;
            Ok::<_, String>(())
        })?;

        Ok(())
    }
    fn layer_from(&self, parent: String, name: String, i: usize) -> Result<Layer, String> {
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
            Ok(parent.child(name, i))
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
            .enumerate()
            .try_for_each(|(i, r)| -> Result<(), String> {
                let [name, params @ ..] = r.list()?.as_slice() else {
                    return Err("Exprected name".to_string());
                };
                match name.atom()? {
                    "defsrc" => {
                        let keymap = parser::parse_keymap(&params.to_vec())?;
                        let mut km: Vec<_> = keymap.iter().collect();
                        km.sort_by_key(|(_, i)| *i);
                        let src = layout.keyboard.source.len();
                        let dst = keymap.len();
                        if src != 0 && src != dst {
                            return Err(format!("Expected {} keys, found {}", src, dst));
                        }
                        layout.keyboard.source = keymap;

                        let src = Layer::from_keyboard(&layout.keyboard.source);
                        layout.layers.insert(src.name.to_string(), src);
                    }
                    "keyboard" => {
                        let [Atom(id)] = params else {
                            return Err("Syntax error".to_string());
                        };
                        layout.keyboard = parser::parse(id)?;
                        let src = Layer::from_keyboard(&layout.keyboard.source);
                        layout.layers.insert(src.name.to_string(), src);
                    }
                    "deflayer" => {
                        let layer = Layer::from_def(params, i)?;
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
                        let mut l = layout.layer_from(layer.parent, layer.name, i)?;
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
                    "defkeymap" => {
                        params.chunks(3).try_for_each(|x| {
                            let [Atom(layer), Atom(keymap), act] = x else {
                                return Err(format!("Syntax error: {:?}", x));
                            };
                            let layer = layout
                                .layers
                                .get_mut(*layer)
                                .ok_or(format!("Layer {:?} not found", layer))?;

                            let keymap: Keymap = keymap
                                .parse()
                                .map_err(|_| format!("Unknown keymap {:?}", keymap))?;

                            let action = Action::from_expr(act)?;

                            layout.keymaps.insert(keymap.clone(), action);
                            layer.keymap = keymap.clone();

                            Ok(())
                        })?;
                    }
                    "defoverride" => {
                        let (name, parent, params) = Layer::get_name(params)?;
                        let mut layer =
                            layout.layer_from(parent.to_string(), name.to_string(), i)?;

                        layer.overrides = params
                            .chunks(2)
                            .map(|x| {
                                let [Atom(src), expr] = x else {
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
                                    .map_err(|k| format!("Expected modifier, found {:?}", k))?;

                                if !layout.keyboard.source.contains_key(key) {
                                    return Err(format!("Key {:?} not in source map", key));
                                }

                                Ok(Override {
                                    key: key.clone(),
                                    action: Action::from_expr(expr)?,
                                    mods: mods.to_vec(),
                                })
                            })
                            .collect::<Result<_, _>>()?;
                        layout.layers.insert(layer.name.to_string(), layer);
                    }
                    "defvial" => layout.keyboard.vial = parse_vial(params)?,

                    _ => return Err(format!("Unexpected {}", name)),
                }
                Ok(())
            })?;
        layout.prepare_layers(&aliases)?;
        Ok(layout)
    }
}

fn resolve_unicode(
    action: &Action,
    lang: &Keymap,
    keymaps: &HashMap<Keymap, Action>,
) -> Result<Action, String> {
    Ok(match action {
        Action::Unicode(ch) => unicode(ch, lang, keymaps)?,
        Action::TapHold(tap, hold) => Action::TapHold(
            Box::new(resolve_unicode(tap, lang, keymaps)?),
            Box::new(resolve_unicode(hold, lang, keymaps)?),
        ),
        Action::Multi(actions) => Action::Multi(
            actions
                .iter()
                .map(|a| resolve_unicode(a, lang, keymaps))
                .collect::<Result<_, _>>()?,
        ),
        Action::Sequence(actions) => Action::Sequence(
            actions
                .iter()
                .map(|a| resolve_unicode(a, lang, keymaps))
                .collect::<Result<_, _>>()?,
        ),
        other => other.clone(),
    })
}

fn check_all_with<T, F>(src: &[T], predicate: F) -> Result<(), &T>
where
    F: Fn(&T) -> bool,
{
    src.iter()
        .find(|item| !predicate(item))
        .map_or(Ok(()), |bad| Err(bad))
}
