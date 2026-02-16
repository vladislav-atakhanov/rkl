use std::collections::{HashMap, HashSet};

use super::config;
use crate::layout::{Action, Layout};
use keys::keys::Key;

fn sorted<'a, K, V, I: Ord, R, F: Fn(&'a K, &'a V) -> (I, R)>(
    map: &'a HashMap<K, V>,
    index_value: F,
) -> Vec<R> {
    let mut source = map.iter().collect::<Vec<_>>();
    source.sort_by_key(|(k, v)| index_value(k, v).0);
    source
        .into_iter()
        .map(|(k, v)| index_value(k, v).1)
        .collect()
}

impl Layout {
    pub fn kanata(&self) -> Result<String, String> {
        let mut lines = vec![];

        let mut source = sorted(&self.keyboard.source, |k, i| (i, k))
            .into_iter()
            .map(key_to_kanata)
            .collect::<Vec<_>>();
        source.insert(0, "defsrc".into());
        lines.push(format!("({})", source.join(" ")));

        let source = self
            .keyboard
            .source
            .iter()
            .map(|(k, i)| (i, k))
            .collect::<HashMap<_, _>>();

        let mut overrides = HashMap::<String, HashSet<String>>::new();

        sorted(&self.layers, |_, l| (l.index, l))
            .into_iter()
            .try_for_each(|l| {
                let actions: Vec<_> = l
                    .keys
                    .iter()
                    .filter_map(|(i, a)| {
                        let Some(key) = source.get(i) else {
                            return Some(Err(format!("Key {:?} not found", i)));
                        };

                        let key = key_to_kanata(key);
                        match action_to_kanata(a) {
                            Ok(action) => {
                                if action != key {
                                    Some(Ok(format!("\t{} {}", key, action)))
                                } else {
                                    None
                                }
                            }

                            e => Some(e),
                        }
                    })
                    .collect::<Result<_, _>>()?;
                lines.push(format!(
                    "(deflayermap ({})\n{}\n)",
                    l.name,
                    actions.join("\n")
                ));

                l.overrides.iter().try_for_each(|o| {
                    let action = action_to_kanata(&o.action)?;
                    let action = if action.starts_with("(") {
                        action
                    } else {
                        format!("({})", action)
                    };
                    let res = format!(
                        "({} {}) {}",
                        o.mods
                            .iter()
                            .map(key_to_kanata)
                            .collect::<Vec<_>>()
                            .join(" "),
                        key_to_kanata(&o.key),
                        action
                    );
                    overrides
                        .entry(res)
                        .or_insert_with(|| HashSet::with_capacity(1))
                        .insert(l.name.clone());
                    Ok::<_, String>(())
                })?;

                Ok::<_, String>(())
            })?;

        if overrides.len() > 0 {
            let all_layers = HashSet::from_iter(self.layers.iter().map(|(n, _)| n.clone()));
            lines.push(format!(
                "(defoverridesv2 \n{}\n)",
                overrides
                    .iter()
                    .map(|(o, l)| {
                        let layers = all_layers
                            .difference(l)
                            .map(|s| s.clone())
                            .collect::<Vec<_>>();
                        format!("\t{} () ({})", o, layers.join(" "))
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        return Ok(lines.join("\n\n"));
    }
}

fn action_to_kanata(action: &Action) -> Result<String, String> {
    Ok(match action {
        Action::Tap(key) => key_to_kanata(key),
        Action::Transparent => "_".into(),
        Action::NoAction => "XX".into(),
        Action::Alias(a) => format!("@{}", a),
        Action::TapHold(tap, hold) => format!(
            "(tap-hold {} {} {} {})",
            config::TAP_HOLD_MS,
            config::TAP_HOLD_MS,
            action_to_kanata(tap)?,
            action_to_kanata(hold)?,
        ),
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
                        return Ok(format!("{}-{}", mods, key_to_kanata(tap)));
                    }
                }
            }
            let res = elems
                .iter()
                .map(action_to_kanata)
                .collect::<Result<Vec<_>, _>>()?;
            let (actions, keys): (Vec<String>, Vec<String>) =
                res.into_iter().partition(|s| s.starts_with("("));

            format!("(multi {} {})", actions.join(" "), keys.join(" "))
        }
        Action::LayerWhileHeld(l) => format!("(layer-while-held {})", l),
        Action::LayerSwitch(l) => format!("(layer-switch {})", l),
        Action::Unicode(_) => todo!(),
        Action::Sequence(actions) => format!(
            "(macro {})",
            actions
                .iter()
                .map(action_to_kanata)
                .collect::<Result<Vec<_>, _>>()?
                .join(" ")
        ),
        Action::Hold(_) | Action::Release(_) => {
            return Err(format!("Action {:?} not in sequence", action));
        }
    })
}

pub fn format_mods(mods: &[&Key]) -> Option<String> {
    let set: HashSet<_> = mods.iter().copied().map(|k| k.clone()).collect();
    let res = set
        .iter()
        .filter_map(|m| {
            Some(match m {
                Key::LeftCtrl => "C",
                Key::RightCtrl => "RC",
                Key::LeftShift => "S",
                Key::RightShift => "RS",
                Key::LeftAlt => "A",
                Key::RightAlt => "A",
                Key::LeftMeta => "M",
                Key::RightMeta => "RM",
                _ => return None,
            })
        })
        .collect::<Vec<_>>();
    if res.len() == set.len() {
        Some(res.join("-"))
    } else {
        None
    }
}

fn key_to_kanata(key: &Key) -> String {
    match key {
        Key::Fn(x) => {
            if *x == 1 {
                "fn".into()
            } else {
                format!("fn{}", x)
            }
        }
        Key::F13 => "f13".into(),
        Key::F14 => "f14".into(),
        Key::F15 => "f15".into(),
        Key::F16 => "f16".into(),
        Key::F17 => "f17".into(),
        Key::F18 => "f18".into(),
        Key::F19 => "f19".into(),
        Key::F20 => "f20".into(),
        Key::F21 => "f21".into(),
        Key::F22 => "f22".into(),
        Key::F23 => "f23".into(),
        Key::F24 => "f24".into(),
        Key::VolumeUp => "volu".into(),
        Key::VolumeDown => "vold".into(),
        Key::VolumeMute => "mute".into(),
        Key::Esc => "esc".into(),
        Key::F1 => "f1".into(),
        Key::F2 => "f2".into(),
        Key::F3 => "f3".into(),
        Key::F4 => "f4".into(),
        Key::F5 => "f5".into(),
        Key::F6 => "f6".into(),
        Key::F7 => "f7".into(),
        Key::F8 => "f8".into(),
        Key::F9 => "f9".into(),
        Key::F10 => "f10".into(),
        Key::F11 => "f11".into(),
        Key::F12 => "f12".into(),
        Key::PrintScreen => "prnt".into(),
        Key::ScrollLock => "sclk".into(),
        Key::Pause => "pause".into(),
        Key::Grave => "grv".into(),

        Key::Zero => "0".into(),
        Key::One => "1".into(),
        Key::Two => "2".into(),
        Key::Three => "3".into(),
        Key::Four => "4".into(),
        Key::Five => "5".into(),
        Key::Six => "6".into(),
        Key::Seven => "7".into(),
        Key::Eight => "8".into(),
        Key::Nine => "9".into(),

        Key::Minus => "-".into(),
        Key::Equal => "=".into(),
        Key::Backspace => "bks".into(),
        Key::Insert => "ins".into(),
        Key::Home => "home".into(),
        Key::PageUp => "pgup".into(),
        Key::PageDown => "pgdn".into(),
        Key::Numlock => "nlck".into(),
        Key::Tab => "tab".into(),
        Key::LeftBracket => "[".into(),
        Key::RightBracket => "]".into(),
        Key::Backslash => "\\".into(),
        Key::Delete => "del".into(),
        Key::End => "end".into(),
        Key::CapsLock => "caps".into(),
        Key::Q => "q".into(),
        Key::W => "w".into(),
        Key::E => "e".into(),
        Key::R => "r".into(),
        Key::T => "t".into(),
        Key::Y => "y".into(),
        Key::U => "u".into(),
        Key::I => "i".into(),
        Key::O => "o".into(),
        Key::P => "p".into(),
        Key::A => "a".into(),
        Key::S => "s".into(),
        Key::D => "d".into(),
        Key::F => "f".into(),
        Key::G => "g".into(),
        Key::H => "h".into(),
        Key::J => "j".into(),
        Key::K => "k".into(),
        Key::L => "l".into(),
        Key::Z => "z".into(),
        Key::X => "x".into(),
        Key::C => "c".into(),
        Key::V => "v".into(),
        Key::B => "b".into(),
        Key::N => "n".into(),
        Key::M => "m".into(),
        Key::Enter => "enter".into(),

        Key::LeftShift => "lsft".into(),
        Key::RightShift => "rsft".into(),
        Key::LeftCtrl => "lctl".into(),
        Key::RightCtrl => "rctl".into(),
        Key::LeftMeta => "lmeta".into(),
        Key::RightMeta => "rmeta".into(),
        Key::LeftAlt => "lalt".into(),
        Key::RightAlt => "ralt".into(),

        Key::Comma => ",".into(),
        Key::Dot => ".".into(),
        Key::Slash => "/".into(),
        Key::Space => "spc".into(),
        Key::Menu => "menu".into(),
        Key::Semicolon => ";".into(),
        Key::Apostrophe => "'".into(),

        Key::Up => "up".into(),
        Key::Down => "down".into(),
        Key::Left => "left".into(),
        Key::Right => "right".into(),

        Key::Kp0 => "kp0".into(),
        Key::Kp1 => "kp1".into(),
        Key::Kp2 => "kp2".into(),
        Key::Kp3 => "kp3".into(),
        Key::Kp4 => "kp4".into(),
        Key::Kp5 => "kp5".into(),
        Key::Kp6 => "kp6".into(),
        Key::Kp7 => "kp7".into(),
        Key::Kp8 => "kp8".into(),
        Key::Kp9 => "kp9".into(),
        Key::KpDot => "kp.".into(),
        Key::KpEnter => "kprt".into(),
        Key::KpPlus => "kp+".into(),
        Key::KpSlash => "kp/".into(),
        Key::KpAsterisk => "kp*".into(),
        Key::KpMinus => "kp-".into(),
        Key::KpEqual => "kp=".into(),

        Key::MediaPlayPause => "pp".into(),
        Key::MouseCursorUp => todo!(),
        Key::MouseCursorDown => todo!(),
        Key::MouseCursorLeft => todo!(),
        Key::MouseCursorRight => todo!(),
        Key::MouseWheelUp => "mwu".into(),
        Key::MouseWheelDown => "mwd".into(),
        Key::MouseWheelLeft => "mwl".into(),
        Key::MouseWheelRight => "mwr".into(),
        Key::MouseButton1 => "mlft".into(),
        Key::MouseButton2 => "mrgt".into(),
        Key::MouseButton3 => "mmid".into(),
        Key::MouseButton4 => "mbck".into(),
        Key::MouseButton5 => "mfwd".into(),
        Key::MouseAcceleration0 => todo!(),
        Key::MouseAcceleration1 => todo!(),
        Key::MouseAcceleration2 => todo!(),
    }
}
