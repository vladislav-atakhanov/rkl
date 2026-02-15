use std::collections::HashMap;

use keys::keys::Key;
use s_expression::Expr::{self, *};

#[derive(Debug, Clone)]
pub enum Action {
    Tap(Key),
    Transparent,
    NoAction,
    Alias(String),
    TapHold(Box<Action>, Box<Action>),
    Multi(Vec<Action>),
    LayerWhileHeld(String),
    LayerSwitch(String),
    Unicode(char),
    Sequence(Vec<Action>),
    #[allow(dead_code)]
    Hold(Key),
    #[allow(dead_code)]
    Release(Key),
}

impl Action {
    pub fn resolve_aliases(&self, aliases: &HashMap<String, Action>) -> Result<Action, String> {
        let res = match self {
            Action::Alias(name) => aliases
                .get(name)
                .map(|a| a.resolve_aliases(aliases))
                .ok_or(format!("Alias @{} not found", name))?,
            Action::TapHold(tap, hold) => Ok(Action::TapHold(
                Box::new(tap.resolve_aliases(aliases)?),
                Box::new(hold.resolve_aliases(aliases)?),
            )),
            Action::Multi(actions) => Ok(Action::Multi(
                actions
                    .iter()
                    .map(|a| a.resolve_aliases(aliases))
                    .collect::<Result<_, _>>()?,
            )),
            _ => Ok(self.clone()),
        };
        return res;
    }

    pub fn layer_while_held_names(&self) -> Vec<&str> {
        match self {
            Action::LayerWhileHeld(name) => vec![name.as_str()],
            Action::TapHold(tap, hold) => {
                let mut v = tap.layer_while_held_names();
                v.extend(hold.layer_while_held_names());
                v
            }
            Action::Multi(actions) | Action::Sequence(actions) => actions
                .iter()
                .flat_map(|a| a.layer_while_held_names())
                .collect(),
            _ => vec![],
        }
    }

    pub fn contains_unicode(&self) -> bool {
        match self {
            Action::Unicode(_) => true,
            Action::TapHold(tap, hold) => tap.contains_unicode() || hold.contains_unicode(),
            Action::Multi(actions) | Action::Sequence(actions) => {
                actions.iter().any(|a| a.contains_unicode())
            }
            _ => false,
        }
    }

    pub fn map_layer_while_held(&mut self, f: &impl Fn(&str) -> Option<String>) {
        match self {
            Action::LayerWhileHeld(name) => {
                if let Some(new) = f(name) {
                    *name = new;
                }
            }
            Action::TapHold(tap, hold) => {
                tap.map_layer_while_held(f);
                hold.map_layer_while_held(f);
            }
            Action::Multi(actions) | Action::Sequence(actions) => {
                actions.iter_mut().for_each(|a| a.map_layer_while_held(f));
            }
            _ => {}
        }
    }

    pub fn from_expr(expr: &Expr) -> Result<Action, String> {
        Ok(match expr {
            Atom(e) => {
                if let Some(d) = e.strip_prefix(".")
                    && !d.is_empty()
                {
                    Self::Unicode(
                        d.chars()
                            .next()
                            .ok_or("Unicode prefix '.' requires a character".to_string())?,
                    )
                } else if let Some(d) = e.strip_prefix("@")
                    && e.len() > 1
                {
                    Action::Alias(d.to_string())
                } else if let keys = e.split("-")
                    && !e.ends_with("-")
                    && !e.starts_with("-")
                    && e.matches("-").count() > 0
                {
                    Action::Multi(
                        keys.map(|key| {
                            key.parse()
                                .map(Action::Tap)
                                .map_err(|_| format!("Unknown key {:?} at {}", key, expr))
                        })
                        .collect::<Result<_, _>>()?,
                    )
                } else {
                    match *e {
                        "X" => Action::NoAction,
                        "_" => Action::Transparent,
                        "lb" => Self::Unicode('('),
                        "rb" => Self::Unicode(')'),
                        k => Action::Tap(k.parse().map_err(|_| format!("Unknown key {:?}", k))?),
                    }
                }
            }
            List(list) => {
                let [Atom(name), params @ ..] = list.as_slice() else {
                    return Err(format!("Unknown action {}", expr));
                };
                match *name {
                    "tap-hold" => {
                        let [tap, hold] = params else {
                            return Err(format!("Syntax error"));
                        };
                        let tap = Self::from_expr(tap)?;
                        let hold = Self::from_expr(hold)?;
                        Action::TapHold(Box::new(tap), Box::new(hold))
                    }
                    "multi" => {
                        let actions: Vec<Action> = params
                            .into_iter()
                            .map(|e| Self::from_expr(e))
                            .collect::<Result<_, _>>()?;
                        Action::Multi(actions)
                    }
                    "layer-while-held" => {
                        let [Atom(name)] = params else {
                            return Err(format!("Syntax error"));
                        };
                        Action::LayerWhileHeld(name.to_string())
                    }
                    "layer-switch" => {
                        let [Atom(name)] = params else {
                            return Err(format!("Syntax error"));
                        };
                        Action::LayerSwitch(name.to_string())
                    }
                    _ => return Err(format!("Unknown action {}", name)),
                }
            }
        })
    }
}
