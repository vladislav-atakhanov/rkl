use std::{collections::HashSet, fmt::Debug};

use keys::keys::Key;
use vitaly::keycodes::{name_to_qid, qid_to_name};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Keycode(pub u16);

impl Debug for Keycode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        qid_to_name(self.0, 6).fmt(f)
    }
}

impl Keycode {
    pub fn from_key(key: &Key, version: u32) -> Result<Self, String> {
        Self::from_name(key_to_string(key).to_string(), version)
    }
    pub fn from_name(name: String, version: u32) -> Result<Self, String> {
        name_to_qid(name.as_str(), version)
            .map(|x| Self(x))
            .map_err(|e| e.to_string())
    }
}

pub fn format_mods(mods: &[&Key]) -> Option<&'static str> {
    let set: HashSet<_> = mods.iter().copied().map(|k| k.clone()).collect();
    if set == HashSet::from([Key::LeftCtrl, Key::LeftShift, Key::LeftAlt, Key::LeftMeta]) {
        Some("HYPR")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftShift, Key::LeftAlt]) {
        Some("MEH")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftAlt, Key::LeftMeta]) {
        Some("LCAG")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftShift]) {
        Some("LCS")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftAlt]) {
        Some("LCA")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftMeta]) {
        Some("LCG")
    } else if set == HashSet::from([Key::RightCtrl, Key::RightMeta]) {
        Some("RCG")
    } else if set == HashSet::from([Key::LeftShift, Key::LeftAlt]) {
        Some("LSA")
    } else if set == HashSet::from([Key::LeftShift, Key::LeftMeta]) {
        Some("LSG")
    } else if set.len() == 1 {
        key_to_mod(mods[0])
    } else {
        None
    }
}

pub fn key_to_mod(key: &Key) -> Option<&'static str> {
    Some(match key {
        Key::LeftAlt => "LALT",
        Key::RightAlt => "RALT",
        Key::LeftCtrl => "LCTL",
        Key::RightCtrl => "RCTL",
        Key::LeftShift => "LSFT",
        Key::RightShift => "RSFT",
        Key::LeftMeta => "LGUI",
        Key::RightMeta => "RGUI",
        _ => return None,
    })
}

pub fn key_to_string(key: &Key) -> &str {
    match key {
        Key::Q => "KC_Q",
        Key::W => "KC_W",
        Key::E => "KC_E",
        Key::R => "KC_R",
        Key::T => "KC_T",
        Key::Y => "KC_Y",
        Key::U => "KC_U",
        Key::I => "KC_I",
        Key::O => "KC_O",
        Key::P => "KC_P",
        Key::A => "KC_A",
        Key::S => "KC_S",
        Key::D => "KC_D",
        Key::F => "KC_F",
        Key::G => "KC_G",
        Key::H => "KC_H",
        Key::J => "KC_J",
        Key::K => "KC_K",
        Key::L => "KC_L",
        Key::Z => "KC_Z",
        Key::X => "KC_X",
        Key::C => "KC_C",
        Key::V => "KC_V",
        Key::B => "KC_B",
        Key::N => "KC_N",
        Key::M => "KC_M",

        Key::Zero => "KC_0",
        Key::One => "KC_1",
        Key::Two => "KC_2",
        Key::Three => "KC_3",
        Key::Four => "KC_4",
        Key::Five => "KC_5",
        Key::Six => "KC_6",
        Key::Seven => "KC_7",
        Key::Eight => "KC_8",
        Key::Nine => "KC_9",

        Key::Fn(_) => "KC_NO",
        Key::F1 => "KC_F1",
        Key::F2 => "KC_F2",
        Key::F3 => "KC_F3",
        Key::F4 => "KC_F4",
        Key::F5 => "KC_F5",
        Key::F6 => "KC_F6",
        Key::F7 => "KC_F7",
        Key::F8 => "KC_F8",
        Key::F9 => "KC_F9",
        Key::F10 => "KC_F10",
        Key::F11 => "KC_F11",
        Key::F12 => "KC_F12",
        Key::F13 => "KC_F13",
        Key::F14 => "KC_F14",
        Key::F15 => "KC_F15",
        Key::F16 => "KC_F16",
        Key::F17 => "KC_F17",
        Key::F18 => "KC_F18",
        Key::F19 => "KC_F19",
        Key::F20 => "KC_F20",
        Key::F21 => "KC_F21",
        Key::F22 => "KC_F22",
        Key::F23 => "KC_F23",
        Key::F24 => "KC_F24",

        Key::VolumeUp => "KC_AUDIO_VOL_UP",
        Key::VolumeDown => "KC_AUDIO_VOL_DOWN",
        Key::VolumeMute => "KC_AUDIO_MUTE",
        Key::Esc => "KC_ESCAPE",

        Key::PrintScreen => "KC_PRINT_SCREEN",
        Key::ScrollLock => "KC_SCROLL_LOCK",
        Key::Pause => "KC_PAUSE",
        Key::Grave => "KC_GRAVE",

        Key::Minus => "KC_MINUS",
        Key::Equal => "KC_EQUAL",
        Key::Backspace => "KC_BACKSPACE",
        Key::Insert => "KC_INSERT",
        Key::Home => "KC_HOME",
        Key::PageUp => "KC_PAGE_UP",
        Key::Numlock => "KC_NUM_LOCK",
        Key::KpSlash => "KC_KP_SLASH",
        Key::KpAsterisk => "KC_KP_ASTERISK",
        Key::KpMinus => "KC_KP_MINUS",
        Key::Tab => "KC_TAB",

        Key::LeftBracket => "KC_LEFT_BRACKET",
        Key::RightBracket => "KC_RIGHT_BRACKET",
        Key::Backslash => "KC_BACKSLASH",
        Key::Delete => "KC_DELETE",
        Key::End => "KC_END",
        Key::PageDown => "KC_PAGE_DOWN",

        Key::KpPlus => "KC_KP_PLUS",
        Key::CapsLock => "KC_CAPS_LOCK",

        Key::Semicolon => "KC_SEMICOLON",
        Key::Apostrophe => "KC_QUOTE",
        Key::Enter => "KC_ENTER",

        Key::LeftShift => "KC_LEFT_SHIFT",

        Key::Comma => "KC_COMMA",
        Key::Dot => "KC_DOT",
        Key::Slash => "KC_SLASH",
        Key::RightShift => "KC_RIGHT_SHIFT",
        Key::Up => "KC_UP",

        Key::Kp0 => "KC_KP_0",
        Key::Kp1 => "KC_KP_1",
        Key::Kp2 => "KC_KP_2",
        Key::Kp3 => "KC_KP_3",
        Key::Kp4 => "KC_KP_4",
        Key::Kp5 => "KC_KP_5",
        Key::Kp6 => "KC_KP_6",
        Key::Kp7 => "KC_KP_7",
        Key::Kp8 => "KC_KP_8",
        Key::Kp9 => "KC_KP_9",

        Key::KpEqual => "KC_KP_EQUAL",
        Key::LeftCtrl => "KC_LEFT_CTRL",
        Key::LeftMeta => "KC_LEFT_GUI",
        Key::LeftAlt => "KC_LEFT_ALT",
        Key::Space => "KC_SPACE",
        Key::RightAlt => "KC_RIGHT_ALT",
        Key::RightMeta => "KC_RIGHT_GUI",
        Key::Menu => "KC_APPLICATION",
        Key::RightCtrl => "KC_RIGHT_CTRL",
        Key::Left => "KC_LEFT",
        Key::Down => "KC_DOWN",
        Key::Right => "KC_RIGHT",
        Key::KpDot => "KC_KP_DOT",
        Key::KpEnter => "KC_KP_ENTER",

        Key::MediaPlayPause => "KC_MEDIA_PLAY_PAUSE",

        Key::MouseCursorUp => "KC_MS_UP",
        Key::MouseCursorDown => "KC_MS_DOWN",
        Key::MouseCursorLeft => "KC_MS_LEFT",
        Key::MouseCursorRight => "KC_MS_RIGHT",
        Key::MouseWheelUp => "KC_WH_U",
        Key::MouseWheelDown => "KC_WH_D",
        Key::MouseWheelLeft => "KC_WH_L",
        Key::MouseWheelRight => "KC_WH_R",
        Key::MouseButton1 => "KC_MS_BTN1",
        Key::MouseButton2 => "KC_MS_BTN2",
        Key::MouseButton3 => "KC_MS_BTN3",
        Key::MouseButton4 => "KC_MS_BTN4",
        Key::MouseButton5 => "KC_MS_BTN5",
        Key::MouseAcceleration0 => "KC_MS_ACCEL0",
        Key::MouseAcceleration1 => "KC_MS_ACCEL1",
        Key::MouseAcceleration2 => "KC_MS_ACCEL2",
    }
}
