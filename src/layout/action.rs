use keys::keys::Key;
use s_expression::Expr::{self, *};

#[derive(Debug, Clone)]
pub enum Action {
    Tap(Key),
    Release(Key),
    Hold(Key),
    Transparent,
    Nil,
    Alias(String),
    MouseWheelUp,
    MouseWheelDown,
    MediaPlayPause,
    TapHold(Box<Action>, Box<Action>),
    Multi(Vec<Action>),
    Macro(Vec<Action>),
    LayerWhileHeld(String),
    LayerSwitch(String),
}

impl Action {
    pub fn unicode(ch: char) -> Result<Self, String> {
        Ok(match ch {
            '=' => Self::Tap(Key::KpEqual),
            '+' => Self::Tap(Key::KpPlus),
            '-' => Self::Tap(Key::KpMinus),
            '*' => Self::Tap(Key::KpAsterisk),
            '/' => Self::Tap(Key::KpSlash),
            '0'..='9' => Self::Tap(Key::from_digit(ch)),

            _ => {
                let mut keys = vec![Self::Hold(Key::LeftAlt)];

                if !ch.is_ascii() || ch.is_control() {
                    return Err(format!("Non-ASCII character {:?}", ch));
                }
                let digits = (ch as u8)
                    .to_string()
                    .chars()
                    .map(|c| {
                        Ok(Self::Tap(match c {
                            '0'..='9' => Key::from_digit(c),
                            _ => return Err(format!("Expected digit, found {:?}", c)),
                        }))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                keys.extend(digits);
                keys.push(Action::Release(Key::LeftAlt));

                Action::Macro(keys)
            }
        })
    }

    pub fn from_expr(expr: &Expr) -> Result<Action, String> {
        Ok(match expr {
            Atom(e) => {
                if let Some(d) = e.strip_prefix(".")
                    && d.len() == 1
                {
                    Self::unicode(d.chars().next().ok_or("Expected symbol".to_string())?)?
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
                        "X" => Action::Nil,
                        "_" => Action::Transparent,
                        "lb" => Self::unicode('(')?,
                        "rb" => Self::unicode(')')?,
                        "MouseWheelUp" | "mwup" => Action::MouseWheelUp,
                        "MouseWheelDown" | "mwdn" => Action::MouseWheelDown,
                        "MediaPlayPause" => Action::MediaPlayPause,
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
