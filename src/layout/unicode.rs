use super::{Action, Keymap};
use s_expression::Expr::*;
use std::collections::HashMap;

use std::sync::OnceLock;

static LANG_CHARS: OnceLock<Result<HashMap<Keymap, HashMap<char, Action>>, String>> =
    OnceLock::new();

fn load_lang_chars() -> Result<HashMap<Keymap, HashMap<char, Action>>, String> {
    let content = format!("({})", include_str!("unicode.rkl"));
    let expr = s_expression::from_str(&content).map_err(|_| "Parse error")?;
    let list = expr.list()?;

    list.iter().try_fold(
        HashMap::<Keymap, HashMap<char, Action>>::with_capacity(list.len()),
        |mut acc, l| {
            let list = l.list()?;
            let [Atom(name), params @ ..] = list.as_slice() else {
                return Err(format!("Name of {} not found", l));
            };
            if *name != "defunicode" {
                return Err(format!("Unknown {:?}", name));
            }

            let [Atom(keymap), params @ ..] = params else {
                return Err(format!("Expected atom, found {:?}", params));
            };

            let keymap = keymap
                .parse::<Keymap>()
                .map_err(|_| format!("Keymap {:?} not found", keymap))?;

            if params.len() % 2 != 0 {
                return Err("Syntax error".to_string());
            }

            acc.insert(
                keymap.clone(),
                params.chunks(2).try_fold(
                    HashMap::with_capacity(params.len() / 2),
                    |mut acc, c| {
                        let [Atom(ch), action] = c else {
                            unreachable!()
                        };

                        let ch = match *ch {
                            "lb" => '(',
                            "rb" => ')',
                            _ => ch
                                .chars()
                                .next()
                                .ok_or(format!("Expected char, found {:?}", ch))?,
                        };

                        let action = Action::from_expr(action)?;
                        acc.insert(ch, action);
                        Ok::<_, String>(acc)
                    },
                )?,
            );

            Ok(acc)
        },
    )
}

pub fn unicode(
    ch: &char,
    lang: &Keymap,
    keymaps: &HashMap<Keymap, Action>,
) -> Result<Action, String> {
    let lang_chars = LANG_CHARS
        .get_or_init(load_lang_chars)
        .as_ref()
        .map_err(|e| e.clone())?;

    if let Some(chars) = lang_chars.get(lang) {
        if let Some(a) = chars.get(ch) {
            return Ok(a.clone());
        }
    };

    if let Some(lang_hotkey) = keymaps.get(lang) {
        for (lang, action) in keymaps.iter() {
            if let Some(chars) = lang_chars.get(lang) {
                if let Some(a) = chars.get(ch) {
                    return Ok(Action::Sequence(
                        [action.clone(), a.clone(), lang_hotkey.clone()].to_vec(),
                    ));
                }
            }
        }
    }

    Ok(Action::Unicode(*ch))
}
