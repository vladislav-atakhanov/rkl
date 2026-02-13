mod macros;
mod tapdance;
use std::fmt::Pointer;

use crate::layout::Action;
use keys::keys::Key;

pub use super::keycode::Keycode;
use super::keycode::{format_mods, key_to_mod, key_to_string};
pub use macros::{Macro, MacroAction};
pub use tapdance::TapDance;

#[derive(Clone)]
pub enum VialAction {
    Keycode(Keycode),
    TapDance(TapDance),
    Macro(Macro),
}

impl std::fmt::Debug for VialAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VialAction::Keycode(x) => x.fmt(f),
            VialAction::TapDance(td) => td.fmt(f),
            VialAction::Macro(v) => v.fmt(f),
        }
    }
}

impl VialAction {
    pub fn tap_hold(tap: Keycode, hold: Keycode) -> Self {
        Self::TapDance(TapDance {
            tap: tap,
            hold: hold,
            double_tap: Keycode(0),
            tap_hold: Keycode(0),
            tapping_term: 200,
        })
    }
    pub fn from_action<'a, L>(
        action: &'a Action,
        layer_by_name: &'a L,
        version: u32,
    ) -> Result<Self, String>
    where
        L: Fn(&str) -> Option<usize>,
    {
        let s = match action {
            Action::NoAction => "KC_NO".to_string(),
            Action::Tap(k) => key_to_string(k).to_string(),
            Action::TapHold(tap, hold) => {
                if let Action::Tap(tap) = tap.as_ref() {
                    match hold.as_ref() {
                        Action::Tap(k) if k.is_modifier() => {
                            return Keycode::from_name(
                                format!(
                                    "{}_T({})",
                                    key_to_mod(k).ok_or(format!("Unreachable {:?}", k))?,
                                    key_to_string(tap)
                                ),
                                version,
                            )
                            .map(Self::Keycode);
                        }
                        Action::LayerSwitch(x) | Action::LayerWhileHeld(x) => {
                            if let Some(l) = layer_by_name(x) {
                                return Keycode::from_name(
                                    format!("LT({},{})", l, key_to_string(tap)),
                                    version,
                                )
                                .map(Self::Keycode);
                            } else {
                                return Err(format!("Layer {} not found", x));
                            }
                        }
                        Action::Multi(actions) => {
                            let mods: Vec<_> = actions
                                .iter()
                                .filter_map(|a| match a {
                                    Action::Tap(k) => key_to_mod(k),
                                    _ => None,
                                })
                                .collect();
                            if mods.len() == actions.len() {
                                return Keycode::from_name(
                                    format!("MT({},{})", mods.join("|"), key_to_string(tap)),
                                    version,
                                )
                                .map(Self::Keycode);
                            }
                        }
                        _ => {}
                    }
                }
                let tap = Self::from_action(tap, layer_by_name, version)?;
                let hold = Self::from_action(hold, layer_by_name, version)?;
                let tap = match tap {
                    VialAction::Keycode(keycode) => keycode,
                    VialAction::TapDance(td) => td.tap,
                    VialAction::Macro(m) => todo!(),
                };
                let hold = match hold {
                    VialAction::Keycode(keycode) => keycode,
                    VialAction::TapDance(td) => td.hold,
                    VialAction::Macro(m) => todo!(),
                };
                return Ok(Self::tap_hold(tap, hold));
            }
            Action::Alias(_) | Action::Unicode(_) => {
                return Err(format!("Action {:?} not implemented", action));
            }
            Action::LayerSwitch(x) => {
                if let Some(l) = layer_by_name(x) {
                    format!("DF({})", l)
                } else {
                    return Err(format!("Layer {} not found", x));
                }
            }
            Action::LayerWhileHeld(x) => {
                if let Some(l) = layer_by_name(x) {
                    format!("MO({})", l)
                } else {
                    return Err(format!("Layer {} not found", x));
                }
            }
            Action::Multi(elems) => {
                let taps: Vec<_> = elems
                    .iter()
                    .map_while(|a| {
                        if let Action::Tap(k) = a {
                            Some(k)
                        } else {
                            None
                        }
                    })
                    .collect();
                if taps.len() == elems.len() {
                    let (mods, keys): (Vec<&Key>, Vec<&Key>) =
                        taps.iter().partition(|k| k.is_modifier());
                    if let [tap] = keys.as_slice() {
                        if let Some(mods) = format_mods(mods.as_slice()) {
                            return Keycode::from_name(
                                format!("{}({})", mods, key_to_string(tap)),
                                version,
                            )
                            .map(Self::Keycode);
                        }
                    }
                }
                todo!("Macro actions");
                // let actions: Vec<_> = elems
                //     .iter()
                //     .map(|a| Self::from_action(a, layer_by_name, version))
                //     .collect::<Result<_, _>>()?;

                // return Ok(Self::Macro(Macro(actions)));
            }
            Action::Transparent => "KC_TRANSPARENT".to_string(),
            Action::Sequence(actions) => {
                return Ok(Self::Macro(Macro(
                    actions
                        .iter()
                        .map(|a| {
                            Ok(match a {
                                Action::Hold(key) => {
                                    MacroAction::Down(Keycode::from_key(key.clone(), version)?)
                                }
                                Action::Release(key) => {
                                    MacroAction::Up(Keycode::from_key(key.clone(), version)?)
                                }

                                a => todo!("{:?}", a), // Self::from_action(a, layer_by_name, version)
                                                       // .map(|a| MacroAction::Tap(a)),
                            })
                        })
                        .collect::<Result<_, String>>()?,
                )));
            }
            Action::Hold(_) | Action::Release(_) => {
                return Err(format!("Action {:?} not in sequence", action));
            }
        };
        Keycode::from_name(s, version).map(Self::Keycode)
    }
}
