use std::collections::HashMap;

use keys::keys::Key;
use log::warn;
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
}

impl Action {
    // pub fn unicode(ch: char) -> Result<Self, String> {
    //     Ok()
    // }

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

    pub fn from_expr(expr: &Expr) -> Result<Action, String> {
        Ok(match expr {
            Atom(e) => {
                if let Some(d) = e.strip_prefix(".")
                    && d.len() > 0
                {
                    Self::Unicode(d.chars().next().unwrap())
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
