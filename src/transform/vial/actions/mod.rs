mod macros;
mod tapdance;
use super::super::config;
pub use super::keycode::Keycode;
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
            tapping_term: config::TAP_HOLD_MS as u16,
        })
    }
}
