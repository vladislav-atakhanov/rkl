use std::str::FromStr;

#[rustfmt::skip]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    Fn(u8),    F13,      F14,     F15,   F16,  F17,  F18, F19,    F20,   F21,  F22,       F23,         F24,                     VolumeUp, VolumeDown, VolumeMute,
    Esc,       F1,       F2,      F3,    F4,   F5,   F6,  F7,     F8,    F9,   F10,       F11,         F12,                     Print,  ScrollLock, Pause,
    Grave,     One,      Two,     Three, Four, Five, Six, Seven,  Eight, Nine, Zero,      Minus,       Equal,        Backspace, Insert, Home,       PageUp,   Numlock, KpSlash, KpAsterisk, KpMinus,
    Tab,       Q,        W,       E,     R,    T,    Y,   U,      I,     O,    P,         LeftBracket, RightBracket, Backslash, Delete, End,        PageDown, Kp7,     Kp8,     Kp9,        KpPlus,
    CapsLock,  A,        S,       D,     F,    G,    H,   J,      K,     L,    Semicolon, Apostrophe,                    Enter,                               Kp4,     Kp5,     Kp6,
    LeftShift, Z,        X,       C,     V,    B,    N,   M,      Comma, Dot,  Slash,                               RightShift,         Up,                   Kp1,     Kp2,     Kp3,        KpEqual,
    LeftCtrl,  LeftMeta, LeftAlt, Space,                                       RightAlt,  RightMeta,   Menu,         RightCtrl, Left,   Down,       Right,    Kp0,              KpDot,      KpEnter,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct KeyIndex(u8);

impl TryFrom<usize> for KeyIndex {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| ())?))
    }
}
impl TryInto<usize> for &KeyIndex {
    type Error = ();
    fn try_into(self) -> Result<usize, Self::Error> {
        Ok(self.0 as usize)
    }
}

impl Key {
    pub fn from_digit(c: char) -> Self {
        match c {
            '0' => Key::Kp0,
            '1' => Key::Kp1,
            '2' => Key::Kp2,
            '3' => Key::Kp3,
            '4' => Key::Kp4,
            '5' => Key::Kp5,
            '6' => Key::Kp6,
            '7' => Key::Kp7,
            '8' => Key::Kp8,
            '9' => Key::Kp9,
            _ => unreachable!(),
        }
    }
}

impl FromStr for Key {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(d) = s.strip_prefix("fn") {
            let num: u8 = d.parse().map_err(|_| ())?;
            return Ok(Self::Fn(num));
        }
        if let Some(d) = s.strip_prefix("KeyFn") {
            let num: u8 = d.parse().map_err(|_| ())?;
            return Ok(Self::Fn(num));
        }
        Ok(match s {
            "KeyEsc" | "esc" => Self::Esc,
            "KeyF1" | "f1" => Self::F1,
            "KeyF2" | "f2" => Self::F2,
            "KeyF3" | "f3" => Self::F3,
            "KeyF4" | "f4" => Self::F4,
            "KeyF5" | "f5" => Self::F5,
            "KeyF6" | "f6" => Self::F6,
            "KeyF7" | "f7" => Self::F7,
            "KeyF8" | "f8" => Self::F8,
            "KeyF9" | "f9" => Self::F9,
            "KeyF10" | "f10" => Self::F10,
            "KeyF11" | "f11" => Self::F11,
            "KeyF12" | "f12" => Self::F12,
            "KeyF13" | "f13" => Self::F13,
            "KeyF14" | "f14" => Self::F14,
            "KeyF15" | "f15" => Self::F15,
            "KeyF16" | "f16" => Self::F16,
            "KeyF17" | "f17" => Self::F17,
            "KeyF18" | "f18" => Self::F18,
            "KeyF19" | "f19" => Self::F19,
            "KeyF20" | "f20" => Self::F20,
            "KeyF21" | "f21" => Self::F21,
            "KeyF22" | "f22" => Self::F22,
            "KeyF23" | "f23" => Self::F23,
            "KeyF24" | "f24" => Self::F24,
            "PrintScreen" => Self::Print,
            "ScrollLock" => Self::ScrollLock,
            "Pause" => Self::Pause,

            "Backquote" | "`" => Self::Grave,
            "Digit1" | "1" => Self::One,
            "Digit2" | "2" => Self::Two,
            "Digit3" | "3" => Self::Three,
            "Digit4" | "4" => Self::Four,
            "Digit5" | "5" => Self::Five,
            "Digit6" | "6" => Self::Six,
            "Digit7" | "7" => Self::Seven,
            "Digit8" | "8" => Self::Eight,
            "Digit9" | "9" => Self::Nine,
            "Digit0" | "0" => Self::Zero,
            "Minus" | "-" => Self::Minus,
            "Equal" | "=" => Self::Equal,
            "Backspace" | "bks" => Self::Backspace,
            "Insert" | "ins" => Self::Insert,
            "Home" | "home" => Self::Home,
            "PageUp" | "pgup" => Self::PageUp,
            "Numlock" => Self::Numlock,

            "Tab" | "tab" => Self::Tab,
            "KeyQ" | "q" => Self::Q,
            "KeyW" | "w" => Self::W,
            "KeyE" | "e" => Self::E,
            "KeyR" | "r" => Self::R,
            "KeyT" | "t" => Self::T,
            "KeyY" | "y" => Self::Y,
            "KeyU" | "u" => Self::U,
            "KeyI" | "i" => Self::I,
            "KeyO" | "o" => Self::O,
            "KeyP" | "p" => Self::P,
            "BracketLeft" | "[" => Self::LeftBracket,
            "BracketRight" | "]" => Self::RightBracket,
            "Backslash" | "\\" => Self::Backslash,
            "Delete" | "del" => Self::Delete,
            "End" | "end" => Self::End,
            "PageDown" | "pgdn" => Self::PageDown,

            "CapsLock" | "caps" => Self::CapsLock,
            "KeyA" | "a" => Self::A,
            "KeyS" | "s" => Self::S,
            "KeyD" | "d" => Self::D,
            "KeyF" | "f" => Self::F,
            "KeyG" | "g" => Self::G,
            "KeyH" | "h" => Self::H,
            "KeyJ" | "j" => Self::J,
            "KeyK" | "k" => Self::K,
            "KeyL" | "l" => Self::L,
            "Semicolon" | ";" => Self::Semicolon,
            "Quote" | "'" => Self::Apostrophe,
            "Enter" | "ent" | "enter" => Self::Enter,

            "KeyZ" | "z" => Self::Z,
            "KeyX" | "x" => Self::X,
            "KeyC" | "c" => Self::C,
            "KeyV" | "v" => Self::V,
            "KeyB" | "b" => Self::B,
            "KeyN" | "n" => Self::N,
            "KeyM" | "m" => Self::M,
            "Comma" | "," => Self::Comma,
            "Period" | "." => Self::Dot,
            "Slash" | "/" => Self::Slash,

            "Numpad0" | "kp0" => Self::Kp0,
            "Numpad1" | "kp1" => Self::Kp1,
            "Numpad2" | "kp2" => Self::Kp2,
            "Numpad3" | "kp3" => Self::Kp3,
            "Numpad4" | "kp4" => Self::Kp4,
            "Numpad5" | "kp5" => Self::Kp5,
            "Numpad6" | "kp6" => Self::Kp6,
            "Numpad7" | "kp7" => Self::Kp7,
            "Numpad8" | "kp8" => Self::Kp8,
            "Numpad9" | "kp9" => Self::Kp9,
            "NumpadPlus" | "kp+" => Self::KpPlus,
            "NumpadEnter" | "kprt" => Self::KpEnter,
            "NumpadDecimal" | "kp." => Self::KpDot,
            "NumpadSlash" | "kp/" => Self::KpSlash,
            "NumpadAsterisk" | "kp*" => Self::KpAsterisk,
            "NumpadMinus" | "kp-" => Self::KpMinus,

            "LeftShift" | "sft" | "lsft" | "LS" | "S" => Self::LeftShift,
            "RightShift" | "rsft" | "RS" => Self::RightShift,

            "LeftCtrl" | "lctl" | "ctl" | "LC" | "C" => Self::LeftCtrl,
            "RightCtrl" | "rctl" | "RC" => Self::RightCtrl,

            "LeftMeta" | "lmeta" | "meta" | "LM" | "M" => Self::LeftMeta,
            "RightMeta" | "rmeta" | "RM" => Self::RightMeta,

            "LeftAlt" | "lalt" | "alt" | "LA" | "A" => Self::LeftAlt,
            "RightAlt" | "ralt" | "RA" => Self::RightAlt,

            "Space" | "spc" => Self::Space,
            "Menu" | "menu" => Self::Menu,

            "ArrowLeft" | "lt" => Self::Left,
            "ArrowDown" | "dn" => Self::Down,
            "ArrowUp" | "up" => Self::Up,
            "ArrowRight" | "rt" => Self::Right,

            "VolumeUp" | "volu" | "vol+" => Self::VolumeUp,
            "VolumeDown" | "vold" | "vol-" => Self::VolumeDown,
            "VolumeMute" | "mute" => Self::VolumeMute,
            _ => return Err(()),
        })
    }
}
